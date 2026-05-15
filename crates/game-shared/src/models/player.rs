use bevy::prelude::*;

/// Marker component for the locally controlled player entity.
#[derive(Component, Debug, Clone)]
pub struct Player;

/// Marker component indicating that the player currently has ground contact.
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct PlayerGrounded;

/// Tuning values used by the player movement controller.
///
/// # Fields
/// - `walk_speed`: Base horizontal speed while walking.
/// - `run_speed`: Horizontal speed while sprinting.
/// - `sneak_speed`: Horizontal speed while sneaking.
/// - `jump_velocity`: Upward velocity applied when jumping.
/// - `ground_acceleration`: Horizontal acceleration while grounded.
/// - `air_acceleration`: Horizontal acceleration while airborne.
#[derive(Component, Debug, Clone)]
pub struct PlayerMovementStats {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub sneak_speed: f32,
    pub jump_velocity: f32,
    pub ground_acceleration: f32,
    pub air_acceleration: f32,
}

impl Default for PlayerMovementStats {
    fn default() -> Self {
        Self {
            walk_speed: 5.2,
            run_speed: 8.6,
            sneak_speed: 2.4,
            jump_velocity: 6.4,
            ground_acceleration: 34.0,
            air_acceleration: 11.0,
        }
    }
}

/// Resource containing movement-related input bindings.
///
/// The client layer mirrors these values from [`crate::config::InputConfig`].
#[derive(Resource, Debug, Clone)]
pub struct PlayerMovementInputConfig {
    pub movement_forward: String,
    pub movement_backward: String,
    pub movement_left: String,
    pub movement_right: String,
    pub movement_jump: String,
    pub movement_sprint: String,
    pub movement_sneak: String,
}

impl Default for PlayerMovementInputConfig {
    fn default() -> Self {
        Self {
            movement_forward: String::from("W"),
            movement_backward: String::from("S"),
            movement_left: String::from("A"),
            movement_right: String::from("D"),
            movement_jump: String::from("Space"),
            movement_sprint: String::from("ShiftLeft"),
            movement_sneak: String::from("CtrlLeft"),
        }
    }
}
