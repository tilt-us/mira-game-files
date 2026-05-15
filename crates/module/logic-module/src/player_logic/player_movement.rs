use avian3d::prelude::{LinearVelocity, ShapeHits};
use bevy::prelude::*;
use game_shared::models::player::{
    Player, PlayerGrounded, PlayerMovementInputConfig, PlayerMovementStats,
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

/// Returns `true` if the configured binding is currently held.
fn is_binding_pressed(binding: &str, keyboard: &ButtonInput<KeyCode>) -> bool {
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
