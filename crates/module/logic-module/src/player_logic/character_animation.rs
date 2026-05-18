use avian3d::prelude::LinearVelocity;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use game_shared::models::character::animation::{
    CharacterAnimationController, CharacterAnimationLoadout, CharacterAnimationNodes,
    CharacterAnimationState,
};
use game_shared::models::player::{Player, PlayerGrounded, PlayerMovementInputConfig, PlayerMovementStats};
use std::time::Duration;
use crate::player_logic::player_movement::is_binding_pressed;

const ANIMATION_KEY_IDLE: &str = "idle";
const ANIMATION_KEY_IDLE_ALT: &str = "idle-02";
const ANIMATION_KEY_SLOW_WALK: &str = "slow_walk";
const ANIMATION_KEY_WALK: &str = "walk";
const ANIMATION_KEY_SPRINT: &str = "sprint";
const ANIMATION_KEY_JUMP: &str = "jump";

/// Initializes animation graphs and default playback for every newly spawned [`AnimationPlayer`].
///
/// The system climbs the parent chain until it finds a party character entity with
/// [`CharacterAnimationLoadout`], then binds graph + transitions to that player.
pub fn setup_character_animation(
    mut commands: Commands,
    parent_query: Query<&ChildOf>,
    loadouts: Query<&CharacterAnimationLoadout>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (animation_player_entity, mut animation_player) in &mut players {
        let mut current_entity = animation_player_entity;
        let mut controller_entity = None;
        let mut loadout = None;

        while let Ok(parent) = parent_query.get(current_entity) {
            current_entity = parent.parent();
            if let Ok(candidate) = loadouts.get(current_entity) {
                controller_entity = Some(current_entity);
                loadout = Some(candidate);
                break;
            }
        }

        let (Some(controller_entity), Some(loadout)) = (controller_entity, loadout) else {
            continue;
        };

        let mut animation_entries = Vec::new();
        for clip in &loadout.clips {
            let clip_handle = asset_server.load(
                GltfAssetLabel::Animation(clip.index).from_asset(loadout.model_asset_path.clone()),
            );
            animation_entries.push((clip.key.as_str(), clip_handle));
        }

        if animation_entries.is_empty() {
            warn!(
                "No animation mappings configured for character entity {:?}",
                controller_entity
            );
            continue;
        }

        let (graph, node_indices) = AnimationGraph::from_clips(
            animation_entries
                .iter()
                .map(|(_, clip_handle)| clip_handle.clone()),
        );
        let graph_handle = graphs.add(graph);

        let mut nodes = CharacterAnimationNodes::default();
        for ((key, _), index) in animation_entries.into_iter().zip(node_indices.into_iter()) {
            match key {
                ANIMATION_KEY_IDLE if nodes.idle.is_none() => nodes.idle = Some(index),
                ANIMATION_KEY_IDLE_ALT if nodes.idle_alt.is_none() => nodes.idle_alt = Some(index),
                ANIMATION_KEY_SLOW_WALK if nodes.slow_walk.is_none() => nodes.slow_walk = Some(index),
                ANIMATION_KEY_WALK if nodes.walk.is_none() => nodes.walk = Some(index),
                ANIMATION_KEY_SPRINT if nodes.sprint.is_none() => nodes.sprint = Some(index),
                ANIMATION_KEY_JUMP if nodes.jump.is_none() => nodes.jump = Some(index),
                _ => {}
            }
        }

        let Some(initial_node) = nodes
            .node_for_state(CharacterAnimationState::Idle)
            .or_else(|| nodes.fallback())
        else {
            warn!(
                "No supported animation keys found for character entity {:?}",
                controller_entity
            );
            continue;
        };

        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut animation_player, initial_node, Duration::ZERO)
            .repeat();

        commands
            .entity(animation_player_entity)
            .insert((AnimationGraphHandle(graph_handle), transitions));

        commands.entity(controller_entity).insert(CharacterAnimationController {
            animation_player_entity,
            nodes,
            current_state: CharacterAnimationState::Idle,
        });
    }
}

/// Updates the animation state for characters, including both the player and companion entities,
/// based on their current movement, grounded state, and proximity or input.
///
/// # Parameters
/// - `keyboard`: A resource that monitors keyboard button input.
/// - `input_config`: A resource that defines the key bindings and movement input configuration for the player.
/// - `player_query`: A query that retrieves the player's `Transform`, `LinearVelocity`,
///   and optional `PlayerGrounded` component. The query is filtered to include only the player entity.
/// - `character_query`: A query for characters that retrieves:
///   - A mutable `CharacterAnimationController` to manage animation states and nodes.
///   - The character's `Transform` for its current world position.
///   - The character's `LinearVelocity` for movement status.
///   - An optional `PlayerMovementStats` for state-specific configuration.
///   - An optional `PlayerGrounded` to check whether the character is on the ground.
///   - A `Has<Player>` marker to distinguish between players and other characters (e.g., companions).
/// - `animation_players`: A query that allows retrieval and modification of the animation player and corresponding
///   animation transitions for a character.
///
/// # Behavior
/// 1. **Player Animation**:
///    - Determines the player's animation state based on velocity, grounded state, and input.
///    - Applies speed adjustments and transitions relevant to the player's animation based on the computed state.
/// 2. **Companion Animation**:
///    - Calculates the distance between the companion and the player.
///    - Adjusts animation states and speed based on their velocity, the player's animation intent, and proximity.
/// 3. **Animation Transitions**:
///    - Checks whether the target animation state differs from the character's current state:
///      - If so, initiates a transition to the target animation node with an interpolative transition duration.
///      - Updates the current state of the `CharacterAnimationController`.
///    - If the character is already in the target state:
///      - Adjusts the animation playback speed based on the current state requirements.
///
/// # Animation Rules
/// - Transitions are handled smoothly with a fixed duration of `140ms` using interpolation.
/// - Animation playback speed is dynamically adjusted depending on the character's state,
///   ensuring state-specific behavior (e.g., walking, running, idle).
///
/// # Notes
/// - If the player entity or its properties (e.g., grounded state or velocity) cannot be resolved,
///   the player's animation defaults to `Idle`.
/// - For companions, fallback logic is applied if the animation node for the target state is not defined.
/// - The function skips entities where animation transitions or nodes cannot be resolved properly.
///
/// # Dependencies
/// - `resolve_player_animation_state`: Computes the target animation state for the player given their input and movement.
/// - `resolve_companion_animation_state`: Computes the target animation state for companions based on proximity and velocity.
/// - `animation_speed_for_state`: Determines the appropriate playback speed for a given animation state.
/// - `CharacterAnimationController::nodes`: Provides access to animation nodes corresponding to character states.
/// - `AnimationTransitions::play`: Initiates an animation transition with the specified configuration.
///
/// # Edge Cases
/// - If a fallback animation node is unavailable, the function will skip transitioning for that entity.
/// - Animation speed adjustments are skipped if the animation is in a non-active or invalid state.
pub fn update_character_animation_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_config: Res<PlayerMovementInputConfig>,
    player_query: Query<(&Transform, &LinearVelocity, Option<&PlayerGrounded>), With<Player>>,
    mut character_query: Query<(
        &mut CharacterAnimationController,
        &Transform,
        &LinearVelocity,
        Option<&PlayerMovementStats>,
        Option<&PlayerGrounded>,
        Has<Player>,
    )>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let (player_position, player_intent) = player_query
        .iter()
        .next()
        .map(|(player_transform, velocity, grounded)| {
            (
                player_transform.translation,
                resolve_player_animation_state(
                    velocity,
                    None,
                    grounded.is_some(),
                    &keyboard,
                    &input_config,
                ),
            )
        })
        .unwrap_or((Vec3::ZERO, CharacterAnimationState::Idle));

    for (mut controller, transform, velocity, movement_stats, is_grounded, is_player) in &mut character_query {
        let target_state = if is_player {
            resolve_player_animation_state(
                velocity,
                movement_stats,
                is_grounded.is_some(),
                &keyboard,
                &input_config,
            )
        } else {
            let distance_to_player = transform.translation.distance(player_position);
            resolve_companion_animation_state(velocity, player_intent, distance_to_player)
        };
        let Some(target_node) = controller
            .nodes
            .node_for_state(target_state)
            .or_else(|| controller.nodes.fallback())
        else {
            continue;
        };

        let Ok((mut animation_player, mut transitions)) =
            animation_players.get_mut(controller.animation_player_entity)
        else {
            continue;
        };

        if target_state != controller.current_state {
            transitions
                .play(
                    &mut animation_player,
                    target_node,
                    Duration::from_millis(140),
                )
                .repeat()
                .set_speed(animation_speed_for_state(target_state));
            controller.current_state = target_state;
            continue;
        }

        if let Some(active) = animation_player.animation_mut(target_node) {
            active.set_speed(animation_speed_for_state(target_state));
        }
    }
}

/// Resolves the appropriate animation state for a character based on input, velocity, and context.
///
/// # Parameters
/// - `velocity`: A reference to the player's current linear velocity.
///   - Used to determine both horizontal and vertical motion speeds.
/// - `movement_stats`: An optional reference to the player's movement statistics, such as walk speed.
///   - Defaults to placeholder values if not provided.
/// - `is_grounded`: A boolean indicating whether the player is on the ground.
/// - `keyboard`: A reference to the player's keyboard input state.
///   - Used to check for movement-related key bindings.
/// - `input_config`: A reference to the player's movement input configurations.
///   - Defines key bindings for actions like forward, backward, sprint, and sneak.
///
/// # Returns
/// A `CharacterAnimationState` enum value that represents the resolved animation state for the player.
///
/// The returned state can be one of the following:
/// - `CharacterAnimationState::Jump`:
///   - When the player is either airborne (`!is_grounded`) or has significant vertical motion.
/// - `CharacterAnimationState::Idle`:
///   - When there is no movement-related input detected from the player.
/// - `CharacterAnimationState::SlowWalk`:
///   - When the player is moving slowly or sneaking (holding the sneak key).
/// - `CharacterAnimationState::Walk`:
///   - When the player is moving at a normal walking speed.
/// - `CharacterAnimationState::Sprint`:
///   - When the player is moving faster than normal (sprinting, not sneaking).
fn resolve_player_animation_state(
    velocity: &LinearVelocity,
    movement_stats: Option<&PlayerMovementStats>,
    is_grounded: bool,
    keyboard: &ButtonInput<KeyCode>,
    input_config: &PlayerMovementInputConfig,
) -> CharacterAnimationState {
    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    let vertical_speed = velocity.y.abs();

    if !is_grounded || vertical_speed > 0.75 {
        return CharacterAnimationState::Jump;
    }

    let is_moving = is_binding_pressed(&input_config.movement_forward, keyboard)
        || is_binding_pressed(&input_config.movement_backward, keyboard)
        || is_binding_pressed(&input_config.movement_left, keyboard)
        || is_binding_pressed(&input_config.movement_right, keyboard);
    if !is_moving {
        return CharacterAnimationState::Idle;
    }

    let is_sneaking = is_binding_pressed(&input_config.movement_sneak, keyboard);
    let is_sprinting = !is_sneaking && is_binding_pressed(&input_config.movement_sprint, keyboard);

    if is_sneaking {
        CharacterAnimationState::SlowWalk
    } else if is_sprinting {
        CharacterAnimationState::Sprint
    } else if horizontal_speed > movement_stats.cloned().unwrap_or_default().walk_speed * 0.6 {
        CharacterAnimationState::Walk
    } else {
        CharacterAnimationState::SlowWalk
    }
}

/// Resolves and determines the appropriate animation state for a companion character
/// based on its movement dynamics, the player's animation state, and the distance to the player.
///
/// # Parameters
/// - `velocity`: A reference to the `LinearVelocity` struct representing the companion's current velocity
///   in terms of x, y, and z axes.
/// - `player_intent`: The current `CharacterAnimationState` representing the player's intended animation state.
/// - `distance_to_player`: A `f32` value representing the horizontal distance between the companion and the player.
///
/// # Returns
/// - `CharacterAnimationState`: The resolved animation state of the companion character based on the following logic:
///   - If the vertical speed (absolute value of y velocity) exceeds `1.1`, the state is set to `Jump`.
///   - If the horizontal speed (calculated using x and z velocity components) is below `0.16`, the state is set to `Idle`.
///   - Otherwise, the animation state is determined based on the player's intent and the distance to the player:
///     - If the player is `Idle`, adjust the companion's animation to a slow walk, walk, or sprint depending on the distance.
///     - If the player's intent is `SlowWalk`, set the companion's state to `SlowWalk`.
///     - If the player's intent is `Walk`, evaluate and adjust the companion's state based on horizontal speed.
///     - If the player's intent is `Sprint`, adjust the companion's state to match a walk or sprint based on horizontal speed.
///     - If the player is in a `Jump` state, adjust the companion's state to a slow walk or walk based on horizontal speed.
///
/// # Notes
/// - The logic prioritizes tighter animation alignment between the companion and the player while maintaining realistic behavior.
/// - Distances less than `5.0` are treated as close proximity, resulting in slower animations, whereas larger distances trigger faster ones.
///
/// # Dependencies
/// - `LinearVelocity`: A type that provides access to an object's velocity components.
/// - `CharacterAnimationState`: An enumeration representing various animation states (e.g., `Idle`, `Walk`, `Sprint`, etc.).
fn resolve_companion_animation_state(
    velocity: &LinearVelocity,
    player_intent: CharacterAnimationState,
    distance_to_player: f32,
) -> CharacterAnimationState {
    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    let vertical_speed = velocity.y.abs();

    if vertical_speed > 1.1 {
        return CharacterAnimationState::Jump;
    }

    if horizontal_speed < 0.16 {
        return CharacterAnimationState::Idle;
    }

    match player_intent {
        CharacterAnimationState::Idle => {
            if distance_to_player < 5.0 {
                CharacterAnimationState::SlowWalk
            } else if distance_to_player < 8.0 {
                CharacterAnimationState::Walk
            } else {
                CharacterAnimationState::Sprint
            }
        }
        CharacterAnimationState::SlowWalk => CharacterAnimationState::SlowWalk,
        CharacterAnimationState::Walk => {
            if horizontal_speed < 1.9 {
                CharacterAnimationState::SlowWalk
            } else {
                CharacterAnimationState::Walk
            }
        }
        CharacterAnimationState::Sprint => {
            if horizontal_speed < 2.0 {
                CharacterAnimationState::Walk
            } else {
                CharacterAnimationState::Sprint
            }
        }
        CharacterAnimationState::Jump => {
            if horizontal_speed < 1.8 {
                CharacterAnimationState::SlowWalk
            } else {
                CharacterAnimationState::Walk
            }
        }
    }
}

fn animation_speed_for_state(state: CharacterAnimationState) -> f32 {
    match state {
        CharacterAnimationState::Idle => 1.0,
        CharacterAnimationState::SlowWalk => 0.9,
        CharacterAnimationState::Walk => 1.0,
        CharacterAnimationState::Sprint => 1.12,
        CharacterAnimationState::Jump => 1.0,
    }
}
