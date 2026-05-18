use avian3d::prelude::{LinearVelocity, ShapeHits};
use bevy::prelude::*;
use game_shared::models::character::Character;
use game_shared::models::player::{
    PartyCompanion, Player, PlayerGrounded, PlayerMovementInputConfig, PlayerMovementStats,
};
use game_shared::utils::input_utils::{KeyType, convert_string_to_key_code, fetch_key_code};
use std::f32::consts::PI;

/// Updates grounded state by evaluating shape cast hits attached to player bodies.
///
/// # Behavior
/// - Inserts [`PlayerGrounded`] when the player shape cast has any contact.
/// - Removes [`PlayerGrounded`] when no contact is present.
pub fn update_player_grounded(
    mut commands: Commands,
    grounded_query: Query<(Entity, &ShapeHits), With<Player>>,
) {
    for (entity, hits) in &grounded_query {
        if hits.is_empty() {
            commands.entity(entity).remove::<PlayerGrounded>();
        } else {
            commands.entity(entity).insert(PlayerGrounded);
        }
    }
}

/// Applies character movement using configured key bindings and camera-relative movement.
///
/// # Controls
/// - Walk: `movement_forward`, `movement_backward`, `movement_left`, `movement_right`
/// - Run: `movement_sprint`
/// - Sneak: `movement_sneak`
/// - Jump: `movement_jump`
///
/// # Notes
/// - Horizontal movement is camera-relative, projected to the XZ plane.
/// - Sprint and sneak are mutually exclusive, with sneak taking priority.
/// - Jump is only applied while grounded.
pub fn player_movement_detect(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    input_config: Res<PlayerMovementInputConfig>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &PlayerMovementStats,
            Has<PlayerGrounded>,
        ),
        With<Player>,
    >,
) {
    let Some(camera_transform) = camera_query.iter().next() else {
        return;
    };

    let mut camera_forward: Vec3 = camera_transform.forward().into();
    camera_forward.y = 0.0;
    camera_forward = camera_forward.normalize_or_zero();

    let mut camera_right: Vec3 = camera_transform.right().into();
    camera_right.y = 0.0;
    camera_right = camera_right.normalize_or_zero();

    for (mut player_transform, mut linear_velocity, stats, is_grounded) in &mut player_query {
        let mut input_axis = Vec2::ZERO;

        if is_binding_pressed(&input_config.movement_forward, &keyboard) {
            input_axis.y += 1.0;
        }
        if is_binding_pressed(&input_config.movement_backward, &keyboard) {
            input_axis.y -= 1.0;
        }
        if is_binding_pressed(&input_config.movement_right, &keyboard) {
            input_axis.x += 1.0;
        }
        if is_binding_pressed(&input_config.movement_left, &keyboard) {
            input_axis.x -= 1.0;
        }

        let move_direction =
            (camera_forward * input_axis.y + camera_right * input_axis.x).normalize_or_zero();

        let is_sneaking = is_binding_pressed(&input_config.movement_sneak, &keyboard);
        let is_sprinting =
            !is_sneaking && is_binding_pressed(&input_config.movement_sprint, &keyboard);

        let target_speed = if is_sneaking {
            stats.sneak_speed
        } else if is_sprinting {
            stats.run_speed
        } else {
            stats.walk_speed
        };

        let target_horizontal_velocity = move_direction * target_speed;
        let current_horizontal_velocity = Vec3::new(linear_velocity.x, 0.0, linear_velocity.z);
        let acceleration = if is_grounded {
            stats.ground_acceleration
        } else {
            stats.air_acceleration
        };
        let step = acceleration * time.delta_secs();
        let next_horizontal_velocity =
            current_horizontal_velocity.move_towards(target_horizontal_velocity, step);

        linear_velocity.x = next_horizontal_velocity.x;
        linear_velocity.z = next_horizontal_velocity.z;

        if is_grounded && is_binding_just_pressed(&input_config.movement_jump, &keyboard) {
            linear_velocity.y = stats.jump_velocity;
        }

        if move_direction != Vec3::ZERO {
            let look_target = player_transform.translation + move_direction;
            player_transform.look_at(look_target, Vec3::Y);
            player_transform.rotate_y(PI);
        }
    }
}

/// Drives party companions to follow the player with relaxed spacing.
///
/// Behavior goals:
/// - Companions stay near the player (not in a strict snake line).
/// - They keep personal distance to player and each other.
/// - They settle around nearby standing spots when the player is idle.
pub fn party_companion_follow(
    time: Res<Time>,
    player_query: Query<
        (&Transform, &LinearVelocity),
        (With<Player>, Without<PartyCompanion>),
    >,
    mut companion_queries: ParamSet<(
        Query<
            (Entity, &Transform, &Character),
            (With<PartyCompanion>, Without<Player>),
        >,
        Query<
            (Entity, &mut Transform, &mut LinearVelocity, &Character),
            (With<PartyCompanion>, Without<Player>),
        >,
    )>,
) {
    let Some((player_transform, player_velocity)) = player_query.iter().next() else {
        return;
    };

    let player_position = player_transform.translation;
    let player_horizontal_velocity = Vec3::new(player_velocity.x, 0.0, player_velocity.z);
    let player_speed = player_horizontal_velocity.length();

    let mut player_forward = if player_speed > 0.2 {
        player_horizontal_velocity.normalize()
    } else {
        let forward: Vec3 = player_transform.forward().into();
        Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero()
    };
    if player_forward == Vec3::ZERO {
        player_forward = Vec3::Z;
    }

    let is_player_moving = player_speed > 0.35;

    let companion_snapshot = {
        let query = companion_queries.p0();
        query
            .iter()
            .map(|(entity, transform, character)| (entity, transform.translation, character.base.id))
            .collect::<Vec<_>>()
    };

    if companion_snapshot.is_empty() {
        return;
    }

    let acceleration = if is_player_moving { 26.0 } else { 18.0 };
    let max_speed = if is_player_moving {
        (player_speed + 2.0).clamp(3.0, 7.0)
    } else {
        3.8
    };
    let step = acceleration * time.delta_secs();

    for (entity, mut transform, mut velocity, character) in &mut companion_queries.p1() {
        let target = companion_slot_target(
            character.base.id,
            player_position,
            player_forward,
            is_player_moving,
        );

        let mut to_target = target - transform.translation;
        to_target.y = 0.0;
        let target_distance = to_target.length();

        let mut separation = Vec3::ZERO;
        for (other_entity, other_position, _) in &companion_snapshot {
            if *other_entity == entity {
                continue;
            }

            let mut away = transform.translation - *other_position;
            away.y = 0.0;
            let distance = away.length();
            let desired_distance = 1.8;
            if distance > 0.001 && distance < desired_distance {
                let pressure = (desired_distance - distance) / desired_distance;
                separation += away.normalize() * pressure;
            }
        }

        let mut away_from_player = transform.translation - player_position;
        away_from_player.y = 0.0;
        let player_distance = away_from_player.length();
        let player_personal_space = 2.0;
        if player_distance > 0.001 && player_distance < player_personal_space {
            let pressure = (player_personal_space - player_distance) / player_personal_space;
            separation += away_from_player.normalize() * pressure * 1.4;
        }

        let mut desired_direction = to_target.normalize_or_zero();
        if separation != Vec3::ZERO {
            desired_direction = (desired_direction + separation * 1.6).normalize_or_zero();
        }

        let desired_speed = if target_distance < 0.6 {
            0.0
        } else if target_distance < 3.2 {
            max_speed * (target_distance / 3.2).clamp(0.2, 1.0)
        } else {
            max_speed
        };

        let desired_horizontal_velocity = desired_direction * desired_speed;
        let current_horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
        let next_horizontal_velocity =
            current_horizontal_velocity.move_towards(desired_horizontal_velocity, step);

        velocity.x = next_horizontal_velocity.x;
        velocity.z = next_horizontal_velocity.z;

        if desired_direction != Vec3::ZERO && desired_speed > 0.15 {
            let look_target = transform.translation + desired_direction;
            transform.look_at(look_target, Vec3::Y);
            transform.rotate_y(PI);
        }
    }
}

/// Calculates the target position for a companion character relative to a player.
///
/// This function determines a position based on the given `companion_id`, player's
/// position, forward direction, and whether the player is currently moving. The companion
/// will follow this target position based on specific logic that incorporates spread angle,
/// side bias, and movement radius.
///
/// # Parameters
///
/// - `companion_id`: A unique identifier for the companion. This is used to generate a
///   pseudo-randomized behavior unique to each companion.
/// - `player_position`: The current position of the player in 3D space, represented as a `Vec3`.
/// - `player_forward`: The forward direction of the player as a unit vector (`Vec3`).
/// - `is_player_moving`: A boolean flag indicating whether the player is currently moving.
///
/// # Returns
///
/// A `Vec3` representing the target position for the companion character.
///
/// # Behavior
///
/// - The function generates a side bias based on the `companion_id` to ensure unique
///   companion behavior.
/// - The spread angle determines how far off-axis the companion's target position
///   can be. It is wider when the player is stationary and narrower when the player is moving.
/// - The base direction for the companion's position is opposite to the player's forward direction
///   when moving and aligned with it when stationary.
/// - The target radius increases slightly when the player is moving to provide a more dynamic
///   movement effect.
/// - The target position calculates a 2D offset based on the base direction and spread angle but
///   retains the player's `y` position to match the height.
fn companion_slot_target(
    companion_id: u64,
    player_position: Vec3,
    player_forward: Vec3,
    is_player_moving: bool,
) -> Vec3 {
    let seed = seed_to_unit_interval(companion_id);
    let side_bias = seed * 2.0 - 1.0;
    let spread_angle = if is_player_moving {
        side_bias * 1.05
    } else {
        side_bias * PI
    };

    let base_direction = if is_player_moving {
        -player_forward
    } else {
        player_forward
    };

    let slot_direction = base_direction.rotate_y(spread_angle).normalize_or_zero();
    let radius = if is_player_moving {
        3.2 + seed * 1.8
    } else {
        2.4 + seed * 2.2
    };

    let mut target = player_position + slot_direction * radius;
    target.y = player_position.y;
    target
}

fn seed_to_unit_interval(seed: u64) -> f32 {
    let hashed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    (hashed as f64 / u64::MAX as f64) as f32
}

/// Checks if a specified key binding is pressed.
///
/// This function evaluates whether the input corresponding to a given binding
/// is currently pressed on the keyboard. The binding can represent a single key,
/// multiple keys (any of which being pressed will trigger a positive result),
/// or a combination of two keys (both of which need to be pressed simultaneously).
///
/// # Arguments
///
/// * `binding` - A string reference representing the key binding. This can map
///   to a single key, a multi-key binding, or a combined key binding.
/// * `keyboard` - A reference to a `ButtonInput<KeyCode>` that is used to check
///   the pressed state of keys on the keyboard.
///
/// # Returns
///
/// * `true` if the binding's corresponding key(s) are pressed.
/// * `false` if the binding is not pressed, is improperly formatted, or its key(s)
///   could not be converted to valid key codes.
///
/// # Key Types
///
/// The `binding` can resolve to one of the following:
/// - `KeyType::SingleKey`: A single key that can be pressed.
/// - `KeyType::MultiKey`: Multiple keys, any one of which being pressed is sufficient.
/// - `KeyType::CombinedKey`: A pair of keys, both of which need to be pressed.
///
/// # Behavior
///
/// 1. If the binding represents a single key:
///    - The function attempts to convert the string representation of the key to a `KeyCode`.
///    - If the conversion is successful, it checks whether the resulting `KeyCode` is pressed.
///    - If the conversion fails, the function returns `false`.
///
/// 2. If the binding represents multiple keys:
///    - The function iterates over the keys in the binding.
///    - If any key converts successfully and is pressed, the function returns `true`.
///    - If no keys are pressed or none could be converted, the function returns `false`.
///
/// 3. If the binding represents a combined key (two keys pressed together):
///    - The function attempts to convert both keys in the pair to `KeyCode`s.
///    - If both conversions succeed and both keys are pressed, the function returns `true`.
///    - If one or both keys cannot be converted or are not pressed, the function returns `false`.
///
/// 4. If the binding is `None` or does not map to any valid key type, the function returns `false`.
pub fn is_binding_pressed(binding: &str, keyboard: &ButtonInput<KeyCode>) -> bool {
    match fetch_key_code(binding) {
        Some(KeyType::SingleKey(key)) => convert_string_to_key_code(&key)
            .map(|key_code| keyboard.pressed(key_code))
            .unwrap_or(false),
        Some(KeyType::MultiKey(keys)) => keys.into_iter().any(|key| {
            convert_string_to_key_code(&key)
                .map(|key_code| keyboard.pressed(key_code))
                .unwrap_or(false)
        }),
        Some(KeyType::CombinedKey((key_a, key_b))) => {
            let Some(key_code_a) = convert_string_to_key_code(&key_a) else {
                return false;
            };
            let Some(key_code_b) = convert_string_to_key_code(&key_b) else {
                return false;
            };
            keyboard.pressed(key_code_a) && keyboard.pressed(key_code_b)
        }
        None => false,
    }
}

/// Returns `true` if the configured binding has been pressed in the current frame.
fn is_binding_just_pressed(binding: &str, keyboard: &ButtonInput<KeyCode>) -> bool {
    match fetch_key_code(binding) {
        Some(KeyType::SingleKey(key)) => convert_string_to_key_code(&key)
            .map(|key_code| keyboard.just_pressed(key_code))
            .unwrap_or(false),
        Some(KeyType::MultiKey(keys)) => keys.into_iter().any(|key| {
            convert_string_to_key_code(&key)
                .map(|key_code| keyboard.just_pressed(key_code))
                .unwrap_or(false)
        }),
        Some(KeyType::CombinedKey((key_a, key_b))) => {
            let Some(key_code_a) = convert_string_to_key_code(&key_a) else {
                return false;
            };
            let Some(key_code_b) = convert_string_to_key_code(&key_b) else {
                return false;
            };
            keyboard.just_pressed(key_code_a) && keyboard.just_pressed(key_code_b)
        }
        None => false,
    }
}
