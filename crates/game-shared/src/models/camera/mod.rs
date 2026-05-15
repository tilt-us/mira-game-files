use bevy::prelude::*;

/// Orbit camera state used to follow and rotate around the active player.
#[derive(Component, Debug, Clone)]
pub struct OrbitFollowCamera {
    pub initialized: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub target_height: f32,
    pub rotation_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for OrbitFollowCamera {
    fn default() -> Self {
        Self {
            initialized: false,
            yaw: 0.0,
            pitch: 0.35,
            distance: 6.0,
            target_height: 1.0,
            rotation_sensitivity: 0.005,
            zoom_sensitivity: 0.6,
            min_pitch: -1.2,
            max_pitch: 1.2,
            min_distance: 2.5,
            max_distance: 12.0,
        }
    }
}

/// Resource tracking whether the in-game menu is open for cursor mode control.
#[derive(Resource, Debug, Clone, Default)]
pub struct MenuCursorState {
    pub is_menu_open: bool,
}
