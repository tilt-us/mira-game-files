use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use game_shared::config::ClientConfigs;
use game_shared::models::camera::OrbitFollowCamera;
use game_shared::models::player::Player;

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
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    client_configs: Option<Res<ClientConfigs>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut OrbitFollowCamera), With<Camera3d>>,
) {
    let Some(player_transform) = player_query.iter().next() else {
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

    let horizontal = orbit.distance * orbit.pitch.cos();
    let offset = Vec3::new(
        horizontal * orbit.yaw.sin(),
        orbit.distance * orbit.pitch.sin(),
        horizontal * orbit.yaw.cos(),
    );

    camera_transform.translation = target + offset;
    camera_transform.look_at(target, Vec3::Y);
}
