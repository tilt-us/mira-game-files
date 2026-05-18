use avian3d::math::Vector;
use avian3d::prelude::{LinearVelocity, ShapeHitData, ShapeHits};
use bevy::ecs::system::IntoSystem;
use bevy::prelude::*;
use game_shared::models::player::{
    PartyCompanion, PartySlot, Player, PlayerGrounded, PlayerMovementInputConfig,
    PlayerMovementStats, PlayerPartyInputConfig,
};
use logic_module::player_logic::player_movement::{
    party_companion_follow, player_movement_detect, swap_active_party_character,
    update_player_grounded,
};
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

    assert!(app
        .world()
        .entity(grounded_entity)
        .contains::<PlayerGrounded>());
    assert!(!app
        .world()
        .entity(airborne_entity)
        .contains::<PlayerGrounded>());
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

#[test]
fn player_movement_detect_uses_multi_key_binding() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig {
        movement_forward: String::from("ArrowUp|W"),
        movement_backward: String::from("S"),
        movement_left: String::from("A"),
        movement_right: String::from("D"),
        movement_jump: String::from("Space"),
        movement_sprint: String::from("ShiftLeft"),
        movement_sneak: String::from("CtrlLeft"),
    });
    app.add_systems(Update, player_movement_detect);
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let player_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
    ));
    let player_entity = player_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::ArrowUp);
    }
    advance_time(&mut app, 1.0);
    app.update();

    let velocity = app
        .world()
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    assert!(velocity.x.is_finite());
    assert!(velocity.z.is_finite());
}

#[test]
fn player_movement_detect_uses_combined_key_binding() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig {
        movement_forward: String::from("ShiftLeft+W"),
        movement_backward: String::from("S"),
        movement_left: String::from("A"),
        movement_right: String::from("D"),
        movement_jump: String::from("CtrlLeft+Space"),
        movement_sprint: String::from("ShiftLeft"),
        movement_sneak: String::from("CtrlLeft"),
    });
    app.add_systems(Update, player_movement_detect);
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let player_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
    ));
    let player_entity = player_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::ShiftLeft);
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::ControlLeft);
        keyboard.press(KeyCode::Space);
    }
    advance_time(&mut app, 1.0 / 60.0);
    app.update();

    let velocity = app
        .world()
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    assert!(velocity.x.is_finite());
    assert!(velocity.z.is_finite());
    assert!(velocity.y > 0.0);
}

#[test]
fn player_movement_detect_ignores_invalid_bindings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig {
        movement_forward: String::from("Unknown+W"),
        movement_backward: String::from("Unknown|S"),
        movement_left: String::from(""),
        movement_right: String::from("Unknown"),
        movement_jump: String::from("Unknown+Space"),
        movement_sprint: String::from("ShiftLeft"),
        movement_sneak: String::from("CtrlLeft"),
    });
    app.add_systems(Update, player_movement_detect);
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let player_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
    ));
    let player_entity = player_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::Space);
    }
    advance_time(&mut app, 1.0 / 30.0);
    app.update();

    let velocity = app
        .world()
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    assert_eq!(velocity.x, 0.0);
    assert_eq!(velocity.z, 0.0);
    assert_eq!(velocity.y, 0.0);
}

#[test]
fn player_movement_detect_returns_without_camera() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, player_movement_detect);

    let player_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vector::new(1.0, 2.0, 3.0)),
        PlayerMovementStats::default(),
    ));
    let player_entity = player_entity.id();

    app.update();

    let velocity = app
        .world()
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    assert_eq!(velocity.x, 1.0);
    assert_eq!(velocity.y, 2.0);
    assert_eq!(velocity.z, 3.0);
}

#[test]
fn player_movement_detect_uses_air_acceleration_when_not_grounded() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, player_movement_detect);
    app.world_mut().spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let stats = PlayerMovementStats {
        walk_speed: 10.0,
        run_speed: 10.0,
        sneak_speed: 2.0,
        jump_velocity: 0.0,
        ground_acceleration: 1000.0,
        air_acceleration: 1.0,
    };
    let player_entity =
        app.world_mut()
            .spawn((Player, Transform::default(), LinearVelocity::ZERO, stats));
    let player_entity = player_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
    }
    advance_time(&mut app, 1.0);
    app.update();

    let velocity = app
        .world()
        .entity(player_entity)
        .get::<LinearVelocity>()
        .expect("expected player velocity");
    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    assert!(horizontal_speed.is_finite());
    assert!(horizontal_speed <= 1.1);
}

#[test]
fn swap_active_party_character_switches_to_requested_slot() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let previous_player = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();
    let requested_player = app
        .world_mut()
        .spawn((PartyCompanion, PartySlot { index: 2 }))
        .id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Digit2);
    }
    app.update();

    assert!(!app.world().entity(previous_player).contains::<Player>());
    assert!(app
        .world()
        .entity(previous_player)
        .contains::<PartyCompanion>());
    assert!(app.world().entity(requested_player).contains::<Player>());
    assert!(!app
        .world()
        .entity(requested_player)
        .contains::<PartyCompanion>());
    assert_eq!(
        app.world()
            .entity(previous_player)
            .get::<PartySlot>()
            .expect("expected previous player slot")
            .index,
        1
    );
    assert_eq!(
        app.world()
            .entity(requested_player)
            .get::<PartySlot>()
            .expect("expected requested player slot")
            .index,
        2
    );
}

#[test]
fn swap_active_party_character_ignores_missing_slot() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let player_entity = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Digit4);
    }
    app.update();

    assert!(app.world().entity(player_entity).contains::<Player>());
    assert!(!app
        .world()
        .entity(player_entity)
        .contains::<PartyCompanion>());
}

#[test]
fn swap_active_party_character_uses_next_slot_binding() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let previous_player = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();
    let next_player = app
        .world_mut()
        .spawn((PartyCompanion, PartySlot { index: 2 }))
        .id();
    app.world_mut()
        .spawn((PartyCompanion, PartySlot { index: 4 }));

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyQ);
    }
    app.update();

    assert!(!app.world().entity(previous_player).contains::<Player>());
    assert!(app.world().entity(next_player).contains::<Player>());
}

#[test]
fn swap_active_party_character_wraps_next_slot_binding() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let wrapped_player = app
        .world_mut()
        .spawn((PartyCompanion, PartySlot { index: 1 }))
        .id();
    app.world_mut()
        .spawn((PartyCompanion, PartySlot { index: 2 }));
    app.world_mut().spawn((Player, PartySlot { index: 4 }));

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyQ);
    }
    app.update();

    assert!(app.world().entity(wrapped_player).contains::<Player>());
}

#[test]
fn swap_active_party_character_next_slot_does_nothing_without_companions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let player_entity = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyQ);
    }
    app.update();

    assert!(app.world().entity(player_entity).contains::<Player>());
    assert!(!app
        .world()
        .entity(player_entity)
        .contains::<PartyCompanion>());
}

#[test]
fn swap_active_party_character_ignores_same_slot_selection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let player_entity = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();
    app.world_mut()
        .spawn((PartyCompanion, PartySlot { index: 2 }));

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Digit1);
    }
    app.update();

    assert!(app.world().entity(player_entity).contains::<Player>());
    assert!(!app
        .world()
        .entity(player_entity)
        .contains::<PartyCompanion>());
}

#[test]
fn swap_active_party_character_returns_when_no_active_player_exists() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let companion_entity = app
        .world_mut()
        .spawn((PartyCompanion, PartySlot { index: 2 }))
        .id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Digit2);
    }
    app.update();

    assert!(app
        .world()
        .entity(companion_entity)
        .contains::<PartyCompanion>());
    assert!(!app.world().entity(companion_entity).contains::<Player>());
}

#[test]
fn swap_active_party_character_supports_third_slot_binding() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let previous_player = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();
    let requested_player = app
        .world_mut()
        .spawn((PartyCompanion, PartySlot { index: 3 }))
        .id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Digit3);
    }
    app.update();

    assert!(app
        .world()
        .entity(previous_player)
        .contains::<PartyCompanion>());
    assert!(app.world().entity(requested_player).contains::<Player>());
}

#[test]
fn swap_active_party_character_does_nothing_without_swap_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerPartyInputConfig::default());
    app.add_systems(Update, swap_active_party_character);

    let player_entity = app.world_mut().spawn((Player, PartySlot { index: 1 })).id();
    let companion_entity = app
        .world_mut()
        .spawn((PartyCompanion, PartySlot { index: 2 }))
        .id();

    app.update();

    assert!(app.world().entity(player_entity).contains::<Player>());
    assert!(app
        .world()
        .entity(companion_entity)
        .contains::<PartyCompanion>());
}

#[test]
fn party_companion_follow_moves_companions_when_player_is_moving() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, party_companion_follow);

    app.world_mut().spawn((
        Player,
        Transform::default(),
        LinearVelocity(Vector::new(3.0, 0.0, 0.0)),
    ));

    let companion_entity = app.world_mut().spawn((
        PartyCompanion,
        PartySlot { index: 2 },
        Transform::from_xyz(-20.0, 0.9, 0.0),
        LinearVelocity::ZERO,
    ));
    let companion_entity = companion_entity.id();

    advance_time(&mut app, 0.5);
    app.update();

    let velocity = app
        .world()
        .entity(companion_entity)
        .get::<LinearVelocity>()
        .expect("expected companion velocity");
    assert!(velocity.x.is_finite());
    assert!(velocity.z.is_finite());
}

#[test]
fn party_companion_follow_applies_player_personal_space() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, party_companion_follow);

    app.world_mut()
        .spawn((Player, Transform::default(), LinearVelocity::ZERO));

    let companion_entity = app.world_mut().spawn((
        PartyCompanion,
        PartySlot { index: 2 },
        Transform::from_xyz(0.4, 0.9, 0.0),
        LinearVelocity::ZERO,
    ));
    let companion_entity = companion_entity.id();

    advance_time(&mut app, 0.5);
    app.update();

    let velocity = app
        .world()
        .entity(companion_entity)
        .get::<LinearVelocity>()
        .expect("expected companion velocity");
    assert!(velocity.x.is_finite());
    assert!(velocity.z.is_finite());
}

#[test]
fn party_companion_follow_applies_companion_separation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, party_companion_follow);

    app.world_mut()
        .spawn((Player, Transform::default(), LinearVelocity::ZERO));

    let companion_left = app.world_mut().spawn((
        PartyCompanion,
        PartySlot { index: 2 },
        Transform::from_xyz(-0.2, 0.9, -3.0),
        LinearVelocity::ZERO,
    ));
    let companion_left = companion_left.id();

    let companion_right = app.world_mut().spawn((
        PartyCompanion,
        PartySlot { index: 3 },
        Transform::from_xyz(0.2, 0.9, -3.0),
        LinearVelocity::ZERO,
    ));
    let companion_right = companion_right.id();

    advance_time(&mut app, 0.5);
    app.update();

    let left_velocity = app
        .world()
        .entity(companion_left)
        .get::<LinearVelocity>()
        .expect("expected left companion velocity");
    let right_velocity = app
        .world()
        .entity(companion_right)
        .get::<LinearVelocity>()
        .expect("expected right companion velocity");

    let left_speed = Vec2::new(left_velocity.x, left_velocity.z).length();
    let right_speed = Vec2::new(right_velocity.x, right_velocity.z).length();
    assert!(left_speed.is_finite());
    assert!(right_speed.is_finite());
}

#[test]
fn party_companion_follow_returns_without_player() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, party_companion_follow);

    let companion_entity = app.world_mut().spawn((
        PartyCompanion,
        PartySlot { index: 2 },
        Transform::from_xyz(4.0, 0.9, 0.0),
        LinearVelocity(Vector::new(0.5, 0.0, 0.0)),
    ));
    let companion_entity = companion_entity.id();

    app.update();

    let velocity = app
        .world()
        .entity(companion_entity)
        .get::<LinearVelocity>()
        .expect("expected companion velocity");
    assert_eq!(velocity.x, 0.5);
}
