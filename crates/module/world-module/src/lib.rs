use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use game_shared::models::world::{TestWorldFloor, TestWorldLight};

/// Spawns a simple test world containing a floor plate and a daylight source.
///
/// # Behavior
/// - Spawns one large static floor mesh with a matching static collider.
/// - Spawns one directional light to provide daylight-style scene lighting.
/// - Skips execution when the required render asset resources are not available.
///   This keeps minimal/headless tests stable.
pub fn spawn_test_world(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    existing_floor: Query<Entity, With<TestWorldFloor>>,
    existing_light: Query<Entity, With<TestWorldLight>>,
) {
    let (Some(meshes), Some(materials)) = (meshes.as_mut(), materials.as_mut()) else {
        return;
    };

    if existing_floor.is_empty() {
        let floor_size = 220.0;
        let floor_thickness = 0.2;
        commands.spawn((
            TestWorldFloor,
            Name::new("Test World Floor"),
            Mesh3d(meshes.add(Cuboid::new(floor_size, floor_thickness, floor_size))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.23, 0.28, 0.24),
                perceptual_roughness: 0.98,
                metallic: 0.0,
                ..default()
            })),
            Transform::from_xyz(0.0, -floor_thickness * 0.5, 0.0),
            RigidBody::Static,
            Collider::cuboid(floor_size, floor_thickness, floor_size),
        ));
    }

    if existing_light.is_empty() {
        commands.spawn((
            TestWorldLight,
            Name::new("Test World Sun"),
            DirectionalLight {
                illuminance: 28_000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 0.65, -0.95, 0.0)),
        ));
    }
}
