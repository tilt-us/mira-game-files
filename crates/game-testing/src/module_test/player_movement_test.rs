use avian3d::math::Vector;
use avian3d::prelude::{LinearVelocity, ShapeHitData, ShapeHits};
use bevy::ecs::system::IntoSystem;
use bevy::prelude::*;
use game_shared::models::player::{
    Player, PlayerGrounded, PlayerMovementInputConfig, PlayerMovementStats,
};
use logic_module::player_logic::player_movement::{player_movement_detect, update_player_grounded};
use std::time::Duration;

fn advance_time(app: &mut App, delta_seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(delta_seconds));
}

#[test]
fn update_player_grounded_inserts_and_removes_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, update_player_grounded);

    let hit_target = app.world_mut().spawn_empty().id();
    let grounded_entity = app
        .world_mut()
        .spawn((
            Player,
            ShapeHits(vec![ShapeHitData {
                entity: hit_target,
                distance: 0.0,
                point1: Vector::ZERO,
                point2: Vector::ZERO,
                normal1: Vector::Y,
                normal2: Vector::Y,
            }]),
        ))
        .id();

    let airborne_entity = app.world_mut().spawn((Player, ShapeHits::default())).id();

    app.update();

    assert!(
        app.world()
            .entity(grounded_entity)
            .contains::<PlayerGrounded>()
    );
    assert!(
        !app.world()
            .entity(airborne_entity)
            .contains::<PlayerGrounded>()
    );
}

#[test]
fn player_movement_detect_applies_jump_when_grounded() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, player_movement_detect);

    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let jump_velocity = 7.5;
    let player_entity = app
        .world_mut()
        .spawn((
            Player,
            PlayerGrounded,
            Transform::default(),
            LinearVelocity::ZERO,
            PlayerMovementStats {
                jump_velocity,
                ..default()
            },
        ))
        .id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Space);
    }
    advance_time(&mut app, 1.0 / 60.0);
    app.update();

    let velocity = app
        .world()
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    assert_eq!(velocity.y, jump_velocity);
}

#[test]
fn player_movement_detect_uses_sneak_speed_over_sprint() {
    let mut world = World::new();
    world.insert_resource(Time::<()>::default());
    world.insert_resource(ButtonInput::<KeyCode>::default());
    world.insert_resource(PlayerMovementInputConfig::default());

    world.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let stats = PlayerMovementStats {
        walk_speed: 1.0,
        run_speed: 9.0,
        sneak_speed: 2.0,
        jump_velocity: 0.0,
        ground_acceleration: 500.0,
        air_acceleration: 500.0,
    };

    let player_entity = world
        .spawn((
            Player,
            PlayerGrounded,
            Transform::default(),
            LinearVelocity::ZERO,
            stats.clone(),
        ))
        .id();

    {
        let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::ShiftLeft);
        keyboard.press(KeyCode::ControlLeft);
    }
    world
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(1.0));

    let mut system = IntoSystem::into_system(player_movement_detect);
    system.initialize(&mut world);
    let _ = system.run((), &mut world);
    system.apply_deferred(&mut world);

    let velocity = world
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    assert!(horizontal_speed > 0.0);
    assert!(horizontal_speed <= stats.sneak_speed + 0.001);
    assert!(velocity.z < 0.0);
}
