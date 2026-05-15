use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use game_shared::models::camera::OrbitFollowCamera;
use game_shared::models::player::Player;
use logic_module::camera_logic::camera_follow::{
    follow_player_orbit_camera, init_orbit_follow_camera,
};

fn spawn_camera(world: &mut World, transform: Transform) -> Entity {
    world
        .spawn((Camera3d::default(), Name::new("Test Camera"), transform))
        .id()
}

#[test]
fn init_orbit_follow_camera_inserts_component_once() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, init_orbit_follow_camera);

    let with_orbit = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            OrbitFollowCamera::default(),
            Name::new("Camera With Orbit"),
        ))
        .id();
    let without_orbit = app
        .world_mut()
        .spawn((Camera3d::default(), Name::new("Camera Without Orbit")))
        .id();

    app.update();

    assert!(
        app.world()
            .entity(with_orbit)
            .contains::<OrbitFollowCamera>()
    );
    assert!(
        app.world()
            .entity(without_orbit)
            .contains::<OrbitFollowCamera>()
    );
}

#[test]
fn follow_player_orbit_camera_applies_rotation_and_zoom() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<MouseMotion>();
    app.add_message::<MouseWheel>();
    app.add_systems(Update, follow_player_orbit_camera);

    app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
    ));

    let _window = app.world_mut().spawn_empty().id();
    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(-2.5, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(24.0, -12.0),
    });
    app.world_mut().write_message(MouseWheel {
        unit: MouseScrollUnit::Line,
        x: 0.0,
        y: 2.0,
        window: camera_entity,
    });

    app.update();

    let camera_transform = app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .expect("expected camera transform");
    let orbit = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component");

    assert!(orbit.initialized);
    assert!(orbit.distance < 6.0);
    assert_ne!(orbit.yaw, 0.0);
    assert_ne!(camera_transform.translation, Vec3::new(-2.5, 2.0, 6.0));
}

#[test]
fn follow_player_orbit_camera_does_nothing_without_player() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<MouseMotion>();
    app.add_message::<MouseWheel>();
    app.add_systems(Update, follow_player_orbit_camera);

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    let before = *app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .expect("expected camera transform");

    app.update();

    let after = *app
        .world()
        .entity(camera_entity)
        .get::<Transform>()
        .expect("expected camera transform");
    assert_eq!(before, after);
}
