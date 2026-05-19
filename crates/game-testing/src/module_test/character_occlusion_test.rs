use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use game_shared::models::camera::{
    CameraOcclusionFade, OcclusionFadeMaterialBinding, OcclusionFadeMaterialOwner,
    OrbitFollowCamera,
};
use game_shared::models::player::Player;
use logic_module::player_logic::character_occlusion::{
    apply_character_occlusion_material_alpha, attach_occlusion_material_owners,
    bind_occlusion_material_instances, update_character_occlusion_targets,
};
use std::time::Duration;

fn init_occlusion_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<StandardMaterial>>();
    app
}

#[test]
fn attach_occlusion_material_owners_only_marks_descendants_of_fade_roots() {
    let mut app = init_occlusion_app();
    app.add_systems(Update, attach_occlusion_material_owners);

    let material_handle = {
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial::default())
    };

    let fade_root = app.world_mut().spawn(CameraOcclusionFade::default()).id();
    let tracked_mesh = app
        .world_mut()
        .spawn((
            MeshMaterial3d(material_handle.clone()),
            Name::new("Tracked Mesh"),
        ))
        .id();
    app.world_mut()
        .entity_mut(fade_root)
        .add_child(tracked_mesh);

    let unrelated_mesh = app
        .world_mut()
        .spawn((MeshMaterial3d(material_handle), Name::new("Unrelated Mesh")))
        .id();

    app.update();

    let tracked_owner = app
        .world()
        .entity(tracked_mesh)
        .get::<OcclusionFadeMaterialOwner>()
        .expect("tracked mesh should receive an owner marker");
    assert_eq!(tracked_owner.root, fade_root);
    assert!(
        app.world()
            .entity(unrelated_mesh)
            .get::<OcclusionFadeMaterialOwner>()
            .is_none(),
        "unrelated mesh should not be marked for occlusion ownership"
    );
}

#[test]
fn bind_occlusion_material_instances_clones_and_binds_per_mesh_material() {
    let mut app = init_occlusion_app();
    app.add_systems(Update, bind_occlusion_material_instances);

    let source_material = {
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial {
            base_color: Color::srgba(0.5, 0.25, 0.75, 0.8),
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })
    };

    let root = app.world_mut().spawn(CameraOcclusionFade::default()).id();
    let mesh_entity = app
        .world_mut()
        .spawn((
            MeshMaterial3d(source_material.clone()),
            OcclusionFadeMaterialOwner { root },
        ))
        .id();

    app.update();

    let entity_ref = app.world().entity(mesh_entity);
    let mesh_material = entity_ref
        .get::<MeshMaterial3d<StandardMaterial>>()
        .expect("mesh should keep a material handle after binding");
    let binding = entity_ref
        .get::<OcclusionFadeMaterialBinding>()
        .expect("mesh should receive an occlusion material binding");

    assert_ne!(mesh_material.0, source_material);
    assert_eq!(binding.material, mesh_material.0);
    assert_eq!(binding.owner_root, root);
    assert!((binding.base_alpha - 0.8).abs() < 0.0001);
    assert_eq!(binding.base_alpha_mode, AlphaMode::Opaque);
}

#[test]
fn update_character_occlusion_targets_only_marks_blocking_entities() {
    let mut app = init_occlusion_app();
    app.add_systems(Update, update_character_occlusion_targets);

    let _camera = app.world_mut().spawn((
        Camera3d::default(),
        OrbitFollowCamera {
            target_height: 0.0,
            occlusion_fade_speed: 40.0,
            ..OrbitFollowCamera::default()
        },
        GlobalTransform::from_translation(Vec3::new(0.0, 1.0, -5.0)),
    ));

    let _player = app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    ));

    let blocking = app.world_mut().spawn((
        CameraOcclusionFade::default(),
        GlobalTransform::from_translation(Vec3::new(0.1, 1.0, -2.0)),
    ));
    let blocking = blocking.id();

    let side = app.world_mut().spawn((
        CameraOcclusionFade::default(),
        GlobalTransform::from_translation(Vec3::new(1.3, 1.0, -2.0)),
    ));
    let side = side.id();

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.2));
    app.update();

    let blocking_fade = app
        .world()
        .entity(blocking)
        .get::<CameraOcclusionFade>()
        .expect("blocking entity should still have fade data");
    let side_fade = app
        .world()
        .entity(side)
        .get::<CameraOcclusionFade>()
        .expect("side entity should still have fade data");

    assert!((blocking_fade.target_alpha - 0.05).abs() < 0.0001);
    assert!(blocking_fade.current_alpha <= 1.0);
    assert_eq!(side_fade.target_alpha, 1.0);
}

#[test]
fn apply_character_occlusion_material_alpha_switches_modes_for_hidden_and_visible_states() {
    let mut app = init_occlusion_app();
    app.add_systems(Update, apply_character_occlusion_material_alpha);

    let root = app.world_mut().spawn(CameraOcclusionFade::default()).id();
    let material_handle = {
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        materials.add(StandardMaterial {
            base_color: Color::srgba(0.9, 0.9, 0.9, 1.0),
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })
    };

    let mesh_entity = app.world_mut().spawn(OcclusionFadeMaterialBinding {
        owner_root: root,
        material: material_handle.clone(),
        base_alpha: 1.0,
        base_alpha_mode: AlphaMode::Opaque,
    });
    let _mesh_entity = mesh_entity.id();

    app.world_mut()
        .entity_mut(root)
        .insert(CameraOcclusionFade {
            target_alpha: 0.05,
            current_alpha: 0.05,
        });
    app.update();

    {
        let materials = app.world().resource::<Assets<StandardMaterial>>();
        let material = materials
            .get(&material_handle)
            .expect("material should exist after first alpha application");
        assert!(material.base_color.alpha() < 0.06);
        assert_eq!(material.alpha_mode, AlphaMode::Blend);
    }

    app.world_mut()
        .entity_mut(root)
        .insert(CameraOcclusionFade {
            target_alpha: 1.0,
            current_alpha: 1.0,
        });
    app.update();

    let materials = app.world().resource::<Assets<StandardMaterial>>();
    let material = materials
        .get(&material_handle)
        .expect("material should exist after second alpha application");
    assert!((material.base_color.alpha() - 1.0).abs() < 0.0001);
    assert_eq!(material.alpha_mode, AlphaMode::Opaque);
}
