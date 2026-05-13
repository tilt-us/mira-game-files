use bevy::prelude::*;

/// Initializes the primary 3D camera for the world scene.
///
/// # Parameters
/// - `commands`: Used to spawn the camera entity when needed.
/// - `existing_camera`: Query used to detect whether a 3D camera already exists.
///
/// # Behavior
/// - If at least one `Camera3d` already exists, this function does nothing.
/// - Otherwise, it spawns a new camera with a default viewing transform.
pub fn init_camera(mut commands: Commands, existing_camera: Query<Entity, With<Camera3d>>) {
    if !existing_camera.is_empty() {
        return;
    }

    commands.spawn((
        Camera3d::default(),
        Name::new("Main Camera"),
        Transform::from_xyz(-2.5, 2.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
