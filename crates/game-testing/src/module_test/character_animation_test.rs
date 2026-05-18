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
