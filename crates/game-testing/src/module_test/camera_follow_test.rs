use avian3d::PhysicsPlugins;
use avian3d::math::Vector;
use avian3d::prelude::{Collider, LinearVelocity, RigidBody};
use bevy::asset::AssetEvent;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::scene::SceneSpawner;
use game_shared::config::ClientConfigs;
use game_shared::models::camera::OrbitFollowCamera;
use game_shared::models::player::{PartyCompanion, Player};
use logic_module::camera_logic::camera_follow::{
    follow_player_orbit_camera, init_orbit_follow_camera,
};
use std::time::Duration;

fn spawn_camera(world: &mut World, transform: Transform) -> Entity {
    world
        .spawn((Camera3d::default(), Name::new("Test Camera"), transform))
        .id()
}

fn init_follow_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(PhysicsPlugins::default());
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<SceneSpawner>();
    app.add_message::<MouseMotion>();
    app.add_message::<MouseWheel>();
    app.add_message::<AssetEvent<Mesh>>();
    app
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
    let mut app = init_follow_app();
    app.add_systems(Update, follow_player_orbit_camera);

    app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
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
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 30.0));
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
    let mut app = init_follow_app();
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

#[test]
fn follow_player_orbit_camera_smoothes_target_when_player_target_changes() {
    let mut app = init_follow_app();
    app.add_systems(Update, follow_player_orbit_camera);

    let player_entity = app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
    ));
    let player_entity = player_entity.id();

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(-2.5, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    app.world_mut()
        .entity_mut(player_entity)
        .insert(GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0)));
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let orbit = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component");

    assert!(orbit.smoothed_target.x > 0.0);
    assert!(orbit.smoothed_target.x < 10.0);
}

#[test]
fn follow_player_orbit_camera_applies_motion_zoom_in_while_moving() {
    let mut app = init_follow_app();
    app.add_systems(Update, follow_player_orbit_camera);

    let player_entity = app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
    ));
    let player_entity = player_entity.id();

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(-2.5, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let initial_distance = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component")
        .smoothed_distance;

    app.world_mut()
        .entity_mut(player_entity)
        .insert(LinearVelocity(Vector::new(7.0, 0.0, 0.0)));

    for _ in 0..12 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
    }

    let moved_distance = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component")
        .smoothed_distance;

    assert!(moved_distance < initial_distance);
}

#[test]
fn follow_player_orbit_camera_reads_mouse_sensitivity_from_client_configs() {
    let mut app = init_follow_app();
    app.insert_resource(ClientConfigs::default());
    app.add_systems(Update, follow_player_orbit_camera);

    app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
    ));

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(-2.5, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    app.world_mut().write_message(MouseMotion {
        delta: Vec2::new(18.0, 0.0),
    });
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let orbit = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component");
    assert_ne!(orbit.yaw, 0.0);
}

#[test]
fn follow_player_orbit_camera_disables_motion_zoom_when_speed_is_zero_or_negative() {
    let mut app = init_follow_app();
    app.add_systems(Update, follow_player_orbit_camera);

    let player_entity = app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
    ));
    let player_entity = player_entity.id();

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(-2.5, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera {
            motion_zoom_speed: 0.0,
            ..OrbitFollowCamera::default()
        });

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let baseline_distance = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component")
        .smoothed_distance;

    app.world_mut()
        .entity_mut(player_entity)
        .insert(LinearVelocity(Vector::new(1000.0, 0.0, 0.0)));

    for _ in 0..6 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
    }

    let orbit = app
        .world()
        .entity(camera_entity)
        .get::<OrbitFollowCamera>()
        .expect("expected orbit component");

    assert!(orbit.smoothed_distance.is_finite());
    assert!((orbit.smoothed_distance - baseline_distance).abs() < 0.0001);
}

#[test]
fn follow_player_orbit_camera_stops_before_static_world_collision() {
    let mut app = init_follow_app();
    app.add_systems(Update, follow_player_orbit_camera);

    app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
    ));

    app.world_mut().spawn((
        Name::new("Blocking Wall"),
        Transform::from_xyz(0.0, 1.4, 2.8),
        GlobalTransform::default(),
        RigidBody::Static,
        Collider::cuboid(8.0, 4.0, 0.9),
    ));

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(0.0, 2.2, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    for _ in 0..4 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
    }

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

    let player_target = Vec3::Y * orbit.target_height;
    let camera_distance = camera_transform.translation.distance(player_target);
    assert!(camera_distance < orbit.smoothed_distance);
}

#[test]
fn follow_player_orbit_camera_ignores_dynamic_entity_collision_blocking() {
    let mut app = init_follow_app();
    app.add_systems(Update, follow_player_orbit_camera);

    app.world_mut().spawn((
        Player,
        GlobalTransform::from_translation(Vec3::ZERO),
        Transform::default(),
        LinearVelocity::ZERO,
    ));

    app.world_mut().spawn((
        PartyCompanion,
        Transform::from_xyz(0.0, 1.0, 3.0),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Collider::capsule(0.4, 1.0),
    ));

    let camera_entity = spawn_camera(
        app.world_mut(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    );
    app.world_mut()
        .entity_mut(camera_entity)
        .insert(OrbitFollowCamera::default());

    for _ in 0..6 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
    }

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

    let player_target = Vec3::Y * orbit.target_height;
    let camera_distance = camera_transform.translation.distance(player_target);
    assert!(camera_distance > orbit.min_distance);
}
