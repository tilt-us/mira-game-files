use avian3d::prelude::LinearVelocity;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use game_shared::config::ClientConfigs;
use game_shared::models::camera::OrbitFollowCamera;
use game_shared::models::player::Player;
use std::f32::consts::{PI, TAU};

/// Ensures that every 3D camera has an [`OrbitFollowCamera`] controller component.
///
/// # Behavior
/// - Adds the controller component to cameras that do not have it yet.
/// - Leaves already configured cameras unchanged.
pub fn init_orbit_follow_camera(
    mut commands: Commands,
    camera_query: Query<Entity, (With<Camera3d>, Without<OrbitFollowCamera>)>,
) {
    for camera_entity in &camera_query {
        commands
            .entity(camera_entity)
            .insert(OrbitFollowCamera::default());
    }
}

/// Updates camera orbit position and zoom so the camera follows the player.
///
/// # Controls
/// - Mouse movement rotates around the player target.
/// - Mouse wheel zooms in and out.
///
/// # Notes
/// - Horizontal and vertical rotation sensitivity are scaled by
///   `InputConfig.mouse_sensitivity_horizontal` and
///   `InputConfig.mouse_sensitivity_vertical`.
/// - The camera always looks at the player target point.
pub fn follow_player_orbit_camera(
    time: Res<Time>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    client_configs: Option<Res<ClientConfigs>>,
    player_query: Query<(&GlobalTransform, Option<&LinearVelocity>), With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut OrbitFollowCamera), With<Camera3d>>,
) {
    let Some((player_transform, player_velocity)) = player_query.iter().next() else {
        return;
    };

    let Some((mut camera_transform, mut orbit)) = camera_query.iter_mut().next() else {
        return;
    };

    let target = player_transform.translation() + Vec3::Y * orbit.target_height;

    if !orbit.initialized {
        let current_offset = camera_transform.translation - target;
        let distance = current_offset.length();

        if distance > f32::EPSILON {
            orbit.distance = distance;
            orbit.yaw = current_offset.x.atan2(current_offset.z);
            orbit.pitch = (current_offset.y / distance)
                .asin()
                .clamp(orbit.min_pitch, orbit.max_pitch);
        }

        orbit.smoothed_yaw = orbit.yaw;
        orbit.smoothed_pitch = orbit.pitch;
        orbit.smoothed_distance = orbit.distance;
        orbit.smoothed_target = target;
        orbit.initialized = true;
    }

    let (horizontal_sensitivity, vertical_sensitivity) = if let Some(configs) = client_configs {
        (
            configs.config_input.mouse_sensitivity_horizontal(),
            configs.config_input.mouse_sensitivity_vertical(),
        )
    } else {
        (1.0, 1.0)
    };

    let mouse_delta = mouse_motion_events
        .read()
        .fold(Vec2::ZERO, |acc, event| acc + event.delta);

    if mouse_delta != Vec2::ZERO {
        orbit.yaw -= mouse_delta.x * orbit.rotation_sensitivity * horizontal_sensitivity;
        orbit.pitch = (orbit.pitch
            - mouse_delta.y * orbit.rotation_sensitivity * vertical_sensitivity)
            .clamp(orbit.min_pitch, orbit.max_pitch);
    }

    let zoom_delta = mouse_wheel_events.read().fold(0.0, |acc, event| {
        let amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y * 0.05,
        };
        acc + amount
    });

    if zoom_delta != 0.0 {
        orbit.distance = (orbit.distance - zoom_delta * orbit.zoom_sensitivity)
            .clamp(orbit.min_distance, orbit.max_distance);
    }

    let player_speed = player_velocity
        .map(|velocity| Vec2::new(velocity.x, velocity.z).length())
        .unwrap_or(0.0);
    let motion_zoom_factor = if orbit.motion_zoom_speed > f32::EPSILON {
        (player_speed / orbit.motion_zoom_speed).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let desired_distance = (orbit.distance - orbit.motion_zoom_in_distance * motion_zoom_factor)
        .clamp(orbit.min_distance, orbit.max_distance);

    let smoothing_alpha = 1.0 - (-orbit.follow_smoothness * time.delta_secs()).exp();
    let rotation_alpha = 1.0 - (-orbit.rotation_smoothness * time.delta_secs()).exp();
    let zoom_alpha = 1.0 - (-orbit.zoom_smoothness * time.delta_secs()).exp();

    orbit.smoothed_yaw = smooth_angle(orbit.smoothed_yaw, orbit.yaw, rotation_alpha);
    orbit.smoothed_pitch = orbit.smoothed_pitch.lerp(orbit.pitch, rotation_alpha);
    orbit.smoothed_distance = orbit.smoothed_distance.lerp(desired_distance, zoom_alpha);
    orbit.smoothed_target = orbit
        .smoothed_target
        .lerp(target, smoothing_alpha.clamp(0.0, 1.0));

    let horizontal = orbit.smoothed_distance * orbit.smoothed_pitch.cos();
    let smoothed_offset = Vec3::new(
        horizontal * orbit.smoothed_yaw.sin(),
        orbit.smoothed_distance * orbit.smoothed_pitch.sin(),
        horizontal * orbit.smoothed_yaw.cos(),
    );

    camera_transform.translation = orbit.smoothed_target + smoothed_offset;
    camera_transform.look_at(orbit.smoothed_target, Vec3::Y);
}

fn smooth_angle(current: f32, target: f32, alpha: f32) -> f32 {
    let delta = wrap_angle(target - current);
    current + delta * alpha.clamp(0.0, 1.0)
}

fn wrap_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}
