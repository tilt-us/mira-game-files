use avian3d::math::Dir;
use avian3d::prelude::{
    Collider, LinearVelocity, RigidBody, ShapeCastConfig, SpatialQuery, SpatialQueryFilter,
};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use game_shared::config::ClientConfigs;
use game_shared::models::camera::OrbitFollowCamera;
use game_shared::models::player::Player;
use std::f32::consts::{PI, TAU};

const LAYER_WORLD: u32 = 1 << 0;

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

/// Updates orbit camera orientation, zoom and world-collision handling.
///
/// # Behavior
/// - Rotates and zooms using mouse input.
/// - Uses only `avian3d` shape casts for camera collision.
/// - Applies a sticky collision constraint to reduce distance oscillation near floors.
pub fn follow_player_orbit_camera(
    time: Res<Time>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    client_configs: Option<Res<ClientConfigs>>,
    spatial_query: SpatialQuery,
    rigid_body_query: Query<&RigidBody>,
    player_query: Query<(Entity, &GlobalTransform, Option<&LinearVelocity>), With<Player>>,
    mut camera_query: Query<(Entity, &mut Transform, &mut OrbitFollowCamera), With<Camera3d>>,
) {
    let Some((player_entity, player_transform, player_velocity)) = player_query.iter().next()
    else {
        return;
    };

    let Some((camera_entity, mut camera_transform, mut orbit)) = camera_query.iter_mut().next()
    else {
        return;
    };

    let target_height = resolve_target_height(orbit.smoothed_distance, &orbit);
    let target = player_transform.translation() + Vec3::Y * target_height;

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
        orbit.collision_smoothed_distance = orbit.distance;
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

    let follow_alpha = 1.0 - (-orbit.follow_smoothness * time.delta_secs()).exp();
    let rotation_alpha = 1.0 - (-orbit.rotation_smoothness * time.delta_secs()).exp();
    let zoom_alpha = 1.0 - (-orbit.zoom_smoothness * time.delta_secs()).exp();

    orbit.smoothed_yaw = smooth_angle(orbit.smoothed_yaw, orbit.yaw, rotation_alpha);
    orbit.smoothed_pitch = orbit.smoothed_pitch.lerp(orbit.pitch, rotation_alpha);
    orbit.smoothed_distance = orbit.smoothed_distance.lerp(desired_distance, zoom_alpha);
    orbit.smoothed_target = orbit
        .smoothed_target
        .lerp(target, follow_alpha.clamp(0.0, 1.0));

    let horizontal = orbit.smoothed_distance * orbit.smoothed_pitch.cos();
    let desired_offset = Vec3::new(
        horizontal * orbit.smoothed_yaw.sin(),
        orbit.smoothed_distance * orbit.smoothed_pitch.sin(),
        horizontal * orbit.smoothed_yaw.cos(),
    );
    let desired_camera_position = orbit.smoothed_target + desired_offset;

    let mut resolved_collision_distance = orbit.smoothed_distance;
    let mut direct_collision_limit = false;
    if let Some((resolved_distance, _)) = resolve_camera_collision(
        &spatial_query,
        &rigid_body_query,
        camera_entity,
        player_entity,
        &orbit,
        orbit.smoothed_target,
        desired_camera_position,
    ) {
        resolved_collision_distance = resolved_distance;
        direct_collision_limit = resolved_distance + 0.1 < orbit.smoothed_distance;
    }

    if direct_collision_limit {
        orbit.collision_is_constrained = true;
    }

    if orbit.collision_is_constrained {
        let collision_alpha =
            1.0 - (-orbit.collision_distance_smoothness * time.delta_secs()).exp();

        if direct_collision_limit {
            let sticky_target = resolved_collision_distance.min(orbit.collision_smoothed_distance);
            orbit.collision_smoothed_distance = orbit
                .collision_smoothed_distance
                .lerp(sticky_target, collision_alpha.clamp(0.0, 1.0));
        }

        let release_gap = orbit.smoothed_distance - orbit.collision_smoothed_distance;
        if !direct_collision_limit && release_gap >= orbit.collision_release_deadzone {
            orbit.collision_is_constrained = false;
        }
    } else {
        let release_alpha = 1.0 - (-orbit.collision_distance_smoothness * time.delta_secs()).exp();
        orbit.collision_smoothed_distance = orbit
            .collision_smoothed_distance
            .lerp(orbit.smoothed_distance, release_alpha.clamp(0.0, 1.0));
    }

    let final_camera_distance = orbit
        .collision_smoothed_distance
        .clamp(orbit.collision_min_distance, orbit.max_distance);

    let camera_direction = desired_offset.normalize_or_zero();
    let final_camera_position = orbit.smoothed_target + camera_direction * final_camera_distance;

    camera_transform.translation = final_camera_position;
    camera_transform.look_at(orbit.smoothed_target, Vec3::Y);
}

fn resolve_camera_collision(
    spatial_query: &SpatialQuery,
    rigid_body_query: &Query<&RigidBody>,
    camera_entity: Entity,
    player_entity: Entity,
    orbit: &OrbitFollowCamera,
    target: Vec3,
    desired_camera_position: Vec3,
) -> Option<(f32, Option<Entity>)> {
    let cast_vector = desired_camera_position - target;
    let desired_distance = cast_vector.length();
    if desired_distance <= f32::EPSILON {
        return None;
    }

    let Ok(direction) = Dir::new(cast_vector.normalize()) else {
        return None;
    };

    let collision_shape = Collider::sphere(orbit.collision_radius);
    let mut cast_config =
        ShapeCastConfig::from_max_distance(desired_distance).with_target_distance(0.02);
    cast_config.ignore_origin_penetration = true;
    cast_config.compute_contact_on_penetration = false;

    let filter = SpatialQueryFilter::from_mask(LAYER_WORLD)
        .with_excluded_entities([camera_entity, player_entity]);

    let hit = spatial_query.cast_shape_predicate(
        &collision_shape,
        target,
        Quat::IDENTITY,
        direction,
        &cast_config,
        &filter,
        &|entity| {
            rigid_body_query
                .get(entity)
                .map(RigidBody::is_static)
                .unwrap_or(false)
        },
    );

    if let Some(hit) = hit {
        let min_distance = orbit.collision_min_distance.min(orbit.min_distance);
        let resolved =
            (hit.distance - orbit.collision_buffer).clamp(min_distance, desired_distance);
        return Some((resolved, Some(hit.entity)));
    }

    Some((desired_distance, None))
}

fn smooth_angle(current: f32, target: f32, alpha: f32) -> f32 {
    let delta = wrap_angle(target - current);
    current + delta * alpha.clamp(0.0, 1.0)
}

fn wrap_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

fn resolve_target_height(current_camera_distance: f32, orbit: &OrbitFollowCamera) -> f32 {
    let blend_start = orbit.target_face_distance;
    let blend_end = orbit.target_face_distance + orbit.target_face_blend_range.max(0.001);
    let face_weight = (1.0 - ((current_camera_distance - blend_start) / (blend_end - blend_start)))
        .clamp(0.0, 1.0);

    orbit
        .target_height
        .lerp(orbit.target_height_face, face_weight)
}
