use avian3d::prelude::LinearVelocity;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use game_shared::models::character::CharacterAnimation;
use game_shared::models::character::animation::{
    CharacterAnimationController, CharacterAnimationLoadout, CharacterAnimationNodes,
    CharacterAnimationState,
};
use game_shared::models::player::{
    Player, PlayerGrounded, PlayerMovementInputConfig, PlayerMovementStats,
};
use logic_module::player_logic::character_animation::{
    setup_character_animation, update_character_animation_state,
};
use std::path::Path;

fn project_assets_path() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .to_string_lossy()
        .to_string()
}

fn test_nodes() -> CharacterAnimationNodes {
    CharacterAnimationNodes {
        idle: Some(AnimationNodeIndex::new(0)),
        idle_alt: Some(AnimationNodeIndex::new(1)),
        slow_walk: Some(AnimationNodeIndex::new(2)),
        walk: Some(AnimationNodeIndex::new(3)),
        sprint: Some(AnimationNodeIndex::new(4)),
        jump: Some(AnimationNodeIndex::new(5)),
    }
}

#[test]
fn setup_character_animation_inserts_graph_and_controller() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.add_systems(Update, setup_character_animation);

    let character_entity = app
        .world_mut()
        .spawn(CharacterAnimationLoadout {
            model_asset_path: String::from("dev/models/lira.glb"),
            clips: vec![
                CharacterAnimation {
                    key: String::from("idle"),
                    index: 1,
                },
                CharacterAnimation {
                    key: String::from("walk"),
                    index: 5,
                },
            ],
        })
        .id();

    let animation_player_entity = app.world_mut().spawn(AnimationPlayer::default()).id();
    app.world_mut()
        .entity_mut(character_entity)
        .add_child(animation_player_entity);

    app.update();

    assert!(
        app.world()
            .entity(animation_player_entity)
            .contains::<AnimationGraphHandle>()
    );
    assert!(
        app.world()
            .entity(animation_player_entity)
            .contains::<AnimationTransitions>()
    );

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller on character root");
    assert_eq!(controller.current_state, CharacterAnimationState::Idle);
    assert_eq!(controller.animation_player_entity, animation_player_entity);
    assert!(controller.nodes.idle.is_some());
}

#[test]
fn setup_character_animation_skips_player_without_loadout_parent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.add_systems(Update, setup_character_animation);

    let animation_player_entity = app.world_mut().spawn(AnimationPlayer::default()).id();
    app.update();

    assert!(
        !app.world()
            .entity(animation_player_entity)
            .contains::<AnimationGraphHandle>()
    );
    assert!(
        !app.world()
            .entity(animation_player_entity)
            .contains::<AnimationTransitions>()
    );
}

#[test]
fn setup_character_animation_skips_empty_clip_mappings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.add_systems(Update, setup_character_animation);

    let character_entity = app
        .world_mut()
        .spawn(CharacterAnimationLoadout {
            model_asset_path: String::from("dev/models/lira.glb"),
            clips: Vec::new(),
        })
        .id();
    let animation_player_entity = app.world_mut().spawn(AnimationPlayer::default()).id();
    app.world_mut()
        .entity_mut(character_entity)
        .add_child(animation_player_entity);

    app.update();

    assert!(
        !app.world()
            .entity(animation_player_entity)
            .contains::<AnimationGraphHandle>()
    );
    assert!(
        !app.world()
            .entity(animation_player_entity)
            .contains::<AnimationTransitions>()
    );
    assert!(
        !app.world()
            .entity(character_entity)
            .contains::<CharacterAnimationController>()
    );
}

#[test]
fn setup_character_animation_skips_when_no_supported_keys_exist() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.add_systems(Update, setup_character_animation);

    let character_entity = app
        .world_mut()
        .spawn(CharacterAnimationLoadout {
            model_asset_path: String::from("dev/models/lira.glb"),
            clips: vec![CharacterAnimation {
                key: String::from("dance"),
                index: 0,
            }],
        })
        .id();
    let animation_player_entity = app.world_mut().spawn(AnimationPlayer::default()).id();
    app.world_mut()
        .entity_mut(character_entity)
        .add_child(animation_player_entity);

    app.update();

    assert!(
        !app.world()
            .entity(animation_player_entity)
            .contains::<AnimationGraphHandle>()
    );
    assert!(
        !app.world()
            .entity(animation_player_entity)
            .contains::<AnimationTransitions>()
    );
    assert!(
        !app.world()
            .entity(character_entity)
            .contains::<CharacterAnimationController>()
    );
}

#[test]
fn setup_character_animation_resolves_loadout_through_multiple_parent_levels() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<AnimationClip>();
    app.init_asset::<AnimationGraph>();
    app.add_systems(Update, setup_character_animation);

    let character_entity = app
        .world_mut()
        .spawn(CharacterAnimationLoadout {
            model_asset_path: String::from("dev/models/lira.glb"),
            clips: vec![CharacterAnimation {
                key: String::from("idle"),
                index: 1,
            }],
        })
        .id();
    let intermediate_entity = app.world_mut().spawn_empty().id();
    let animation_player_entity = app.world_mut().spawn(AnimationPlayer::default()).id();
    app.world_mut()
        .entity_mut(character_entity)
        .add_child(intermediate_entity);
    app.world_mut()
        .entity_mut(intermediate_entity)
        .add_child(animation_player_entity);

    app.update();

    assert!(
        app.world()
            .entity(animation_player_entity)
            .contains::<AnimationGraphHandle>()
    );
    assert!(
        app.world()
            .entity(animation_player_entity)
            .contains::<AnimationTransitions>()
    );
    assert!(
        app.world()
            .entity(character_entity)
            .contains::<CharacterAnimationController>()
    );
}

#[test]
fn update_character_animation_state_keeps_player_idle_without_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Walk,
        },
    ));
    let character_entity = character_entity.id();

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Idle);
}

#[test]
fn update_character_animation_state_transitions_player_to_jump_when_airborne() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        Transform::default(),
        LinearVelocity(Vec3::new(0.0, 1.2, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let character_entity = character_entity.id();

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Jump);
}

#[test]
fn update_character_animation_state_switches_player_to_sprint_on_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vec3::new(8.0, 0.0, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let character_entity = character_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::ShiftLeft);
    }

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Sprint);
}

#[test]
fn update_character_animation_state_keeps_state_when_no_supported_nodes_exist() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: CharacterAnimationNodes::default(),
            current_state: CharacterAnimationState::Sprint,
        },
    ));
    let character_entity = character_entity.id();

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Sprint);
}

#[test]
fn update_character_animation_state_keeps_state_when_animation_player_entity_is_missing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let stale_animation_player_entity = app.world_mut().spawn_empty().id();
    assert!(
        app.world_mut().despawn(stale_animation_player_entity),
        "expected despawn to succeed"
    );

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity: stale_animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Walk,
        },
    ));
    let character_entity = character_entity.id();

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Walk);
}

#[test]
fn update_character_animation_state_starts_missing_active_animation_for_same_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let nodes = test_nodes();
    let idle_node = nodes.idle.expect("expected idle node");
    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes,
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let character_entity = character_entity.id();

    app.update();

    let animation_player = app
        .world()
        .entity(animation_player_entity)
        .get::<AnimationPlayer>()
        .expect("expected animation player");
    assert!(animation_player.animation(idle_node).is_some());

    app.update();

    let animation_player = app
        .world()
        .entity(animation_player_entity)
        .get::<AnimationPlayer>()
        .expect("expected animation player");
    let active_idle = animation_player
        .animation(idle_node)
        .expect("expected active idle animation");
    assert_eq!(active_idle.speed(), 1.0);

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Idle);
}

#[test]
fn update_character_animation_state_uses_slow_walk_for_sneaking_player() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vec3::new(3.0, 0.0, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let character_entity = character_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::ControlLeft);
    }

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::SlowWalk);
}

#[test]
fn update_character_animation_state_uses_walk_for_moving_player_without_sprint_or_sneak() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vec3::new(4.0, 0.0, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let character_entity = character_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
    }

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::Walk);
}

#[test]
fn update_character_animation_state_uses_slow_walk_for_low_speed_player_motion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let animation_player_entity = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let animation_player_entity = animation_player_entity.id();

    let character_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vec3::new(1.0, 0.0, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let character_entity = character_entity.id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
    }

    app.update();

    let controller = app
        .world()
        .entity(character_entity)
        .get::<CharacterAnimationController>()
        .expect("expected animation controller");
    assert_eq!(controller.current_state, CharacterAnimationState::SlowWalk);
}

#[test]
fn update_character_animation_state_uses_companion_speed_bands_for_non_idle_player_intent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let player_animation_player = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let player_animation_player = player_animation_player.id();
    app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vec3::new(8.0, 0.0, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity: player_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::ShiftLeft);
    }

    let slow_animation_player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), AnimationTransitions::new()))
        .id();
    let slow_player = app.world_mut().spawn((
        Transform::from_xyz(2.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(1.2, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: slow_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let slow_player = slow_player.id();

    let walk_animation_player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), AnimationTransitions::new()))
        .id();
    let walk_player = app.world_mut().spawn((
        Transform::from_xyz(3.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(3.1, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: walk_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let walk_player = walk_player.id();

    let sprint_animation_player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), AnimationTransitions::new()))
        .id();
    let sprint_player = app.world_mut().spawn((
        Transform::from_xyz(4.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(5.3, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: sprint_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let sprint_player = sprint_player.id();

    app.update();

    let slow_state = app
        .world()
        .entity(slow_player)
        .get::<CharacterAnimationController>()
        .expect("expected slow companion controller")
        .current_state;
    let walk_state = app
        .world()
        .entity(walk_player)
        .get::<CharacterAnimationController>()
        .expect("expected walk companion controller")
        .current_state;
    let sprint_state = app
        .world()
        .entity(sprint_player)
        .get::<CharacterAnimationController>()
        .expect("expected sprint companion controller")
        .current_state;

    assert_eq!(slow_state, CharacterAnimationState::SlowWalk);
    assert_eq!(walk_state, CharacterAnimationState::Walk);
    assert_eq!(sprint_state, CharacterAnimationState::Sprint);
}

#[test]
fn update_character_animation_state_sets_idle_for_very_slow_companion_and_jump_for_vertical_motion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let player_animation_player = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let player_animation_player = player_animation_player.id();
    app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::default(),
        LinearVelocity(Vec3::new(6.0, 0.0, 0.0)),
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity: player_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::KeyW);
        keyboard.press(KeyCode::ShiftLeft);
    }

    let idle_animation_player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), AnimationTransitions::new()))
        .id();
    let idle_companion = app.world_mut().spawn((
        Transform::from_xyz(2.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(0.01, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: idle_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Walk,
        },
    ));
    let idle_companion = idle_companion.id();

    let jump_animation_player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), AnimationTransitions::new()))
        .id();
    let jump_companion = app.world_mut().spawn((
        Transform::from_xyz(3.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(0.5, 1.2, 0.0)),
        CharacterAnimationController {
            animation_player_entity: jump_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let jump_companion = jump_companion.id();

    app.update();

    let idle_state = app
        .world()
        .entity(idle_companion)
        .get::<CharacterAnimationController>()
        .expect("expected idle companion controller")
        .current_state;
    let jump_state = app
        .world()
        .entity(jump_companion)
        .get::<CharacterAnimationController>()
        .expect("expected jump companion controller")
        .current_state;

    assert_eq!(idle_state, CharacterAnimationState::Idle);
    assert_eq!(jump_state, CharacterAnimationState::Jump);
}

#[test]
fn update_character_animation_state_uses_distance_thresholds_for_idle_player_companions() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(PlayerMovementInputConfig::default());
    app.add_systems(Update, update_character_animation_state);

    let player_animation_player = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let player_animation_player = player_animation_player.id();
    let player_entity = app.world_mut().spawn((
        Player,
        PlayerGrounded,
        Transform::from_xyz(0.0, 0.0, 0.0),
        LinearVelocity::ZERO,
        PlayerMovementStats::default(),
        CharacterAnimationController {
            animation_player_entity: player_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Walk,
        },
    ));
    let _player_entity = player_entity.id();

    let companion_a_animation_player = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let companion_a_animation_player = companion_a_animation_player.id();
    let companion_a = app.world_mut().spawn((
        Transform::from_xyz(4.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(1.0, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: companion_a_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let companion_a = companion_a.id();

    let companion_b_animation_player = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let companion_b_animation_player = companion_b_animation_player.id();
    let companion_b = app.world_mut().spawn((
        Transform::from_xyz(7.0, 0.0, 0.0),
        LinearVelocity(Vec3::new(2.2, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: companion_b_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let companion_b = companion_b.id();

    let companion_c_animation_player = app.world_mut().spawn((
        AnimationPlayer::default(),
        AnimationTransitions::new(),
    ));
    let companion_c_animation_player = companion_c_animation_player.id();
    let companion_c = app.world_mut().spawn((
        Transform::from_xyz(9.5, 0.0, 0.0),
        LinearVelocity(Vec3::new(5.0, 0.0, 0.0)),
        CharacterAnimationController {
            animation_player_entity: companion_c_animation_player,
            nodes: test_nodes(),
            current_state: CharacterAnimationState::Idle,
        },
    ));
    let companion_c = companion_c.id();

    app.update();

    let state_a = app
        .world()
        .entity(companion_a)
        .get::<CharacterAnimationController>()
        .expect("expected companion a controller")
        .current_state;
    let state_b = app
        .world()
        .entity(companion_b)
        .get::<CharacterAnimationController>()
        .expect("expected companion b controller")
        .current_state;
    let state_c = app
        .world()
        .entity(companion_c)
        .get::<CharacterAnimationController>()
        .expect("expected companion c controller")
        .current_state;

    assert_eq!(state_a, CharacterAnimationState::SlowWalk);
    assert_eq!(state_b, CharacterAnimationState::Walk);
    assert_eq!(state_c, CharacterAnimationState::Sprint);
}
