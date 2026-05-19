use bevy::prelude::*;

/// Orbit camera state used to follow and rotate around the active player.
#[derive(Component, Debug, Clone)]
pub struct OrbitFollowCamera {
    pub initialized: bool,
    pub smoothed_target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub smoothed_yaw: f32,
    pub smoothed_pitch: f32,
    pub smoothed_distance: f32,
    pub target_height: f32,
    pub follow_smoothness: f32,
    pub rotation_smoothness: f32,
    pub zoom_smoothness: f32,
    pub motion_zoom_in_distance: f32,
    pub motion_zoom_speed: f32,
    pub rotation_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub target_height_face: f32,
    pub target_face_distance: f32,
    pub target_face_blend_range: f32,
    pub collision_radius: f32,
    pub collision_buffer: f32,
    pub collision_min_distance: f32,
    pub collision_distance_smoothness: f32,
    pub collision_distance_deadzone: f32,
    pub collision_smoothed_distance: f32,
    pub collision_release_deadzone: f32,
    pub collision_is_constrained: bool,
    pub occlusion_alpha_near: f32,
    pub occlusion_alpha_far: f32,
    pub occlusion_alpha_near_distance: f32,
    pub occlusion_alpha_far_distance: f32,
    pub occlusion_fade_speed: f32,
    pub occlusion_player_distance: f32,
}

impl Default for OrbitFollowCamera {
    fn default() -> Self {
        Self {
            initialized: false,
            smoothed_target: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.35,
            distance: 6.0,
            smoothed_yaw: 0.0,
            smoothed_pitch: 0.35,
            smoothed_distance: 6.0,
            target_height: 0.55,
            follow_smoothness: 9.0,
            rotation_smoothness: 14.0,
            zoom_smoothness: 11.0,
            motion_zoom_in_distance: 1.2,
            motion_zoom_speed: 7.0,
            rotation_sensitivity: 0.005,
            zoom_sensitivity: 0.6,
            min_pitch: -1.2,
            max_pitch: 1.2,
            min_distance: 2.5,
            max_distance: 12.0,
            target_height_face: 1.15,
            target_face_distance: 2.0,
            target_face_blend_range: 0.55,
            collision_radius: 0.24,
            collision_buffer: 0.08,
            collision_min_distance: 0.65,
            collision_distance_smoothness: 26.0,
            collision_distance_deadzone: 0.025,
            collision_smoothed_distance: 6.0,
            collision_release_deadzone: 0.2,
            collision_is_constrained: false,
            occlusion_alpha_near: 0.2,
            occlusion_alpha_far: 0.7,
            occlusion_alpha_near_distance: 0.9,
            occlusion_alpha_far_distance: 2.2,
            occlusion_fade_speed: 12.0,
            occlusion_player_distance: 1.45,
        }
    }
}

/// Runtime opacity target used for camera-driven occlusion fading.
#[derive(Component, Debug, Clone, Copy)]
pub struct CameraOcclusionFade {
    pub target_alpha: f32,
    pub current_alpha: f32,
}

impl Default for CameraOcclusionFade {
    fn default() -> Self {
        Self {
            target_alpha: 1.0,
            current_alpha: 1.0,
        }
    }
}

/// Resource tracking whether the in-game menu is open for cursor mode control.
#[derive(Resource, Debug, Clone, Default)]
pub struct MenuCursorState {
    pub is_menu_open: bool,
}
