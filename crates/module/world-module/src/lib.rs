use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use game_shared::models::world::{TestWorldFloor, TestWorldLight, TestWorldObstacle, TestWorldWall};

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
    existing_walls: Query<Entity, With<TestWorldWall>>,
    existing_obstacles: Query<Entity, With<TestWorldObstacle>>,
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
                base_color: Color::srgb(0.16, 0.17, 0.18),
                perceptual_roughness: 0.98,
                metallic: 0.0,
                ..default()
            })),
            Transform::from_xyz(0.0, -floor_thickness * 0.5, 0.0),
            RigidBody::Static,
            Collider::cuboid(floor_size, floor_thickness, floor_size),
        ));
    }

    if existing_walls.is_empty() {
        let room_half_extent = 38.0;
        let wall_height = 5.0;
        let wall_thickness = 1.4;
        let wall_color = materials.add(StandardMaterial {
            base_color: Color::srgb(0.44, 0.46, 0.49),
            perceptual_roughness: 0.93,
            metallic: 0.0,
            ..default()
        });

        let walls = [
            (
                "Test World Wall North",
                Vec3::new(room_half_extent * 2.0, wall_height, wall_thickness),
                Vec3::new(0.0, wall_height * 0.5, room_half_extent),
            ),
            (
                "Test World Wall South",
                Vec3::new(room_half_extent * 2.0, wall_height, wall_thickness),
                Vec3::new(0.0, wall_height * 0.5, -room_half_extent),
            ),
            (
                "Test World Wall East",
                Vec3::new(wall_thickness, wall_height, room_half_extent * 2.0),
                Vec3::new(room_half_extent, wall_height * 0.5, 0.0),
            ),
            (
                "Test World Wall West",
                Vec3::new(wall_thickness, wall_height, room_half_extent * 2.0),
                Vec3::new(-room_half_extent, wall_height * 0.5, 0.0),
            ),
        ];

        for (name, size, position) in walls {
            commands.spawn((
                TestWorldWall,
                Name::new(name),
                Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                MeshMaterial3d(wall_color.clone()),
                Transform::from_translation(position),
                RigidBody::Static,
                Collider::cuboid(size.x, size.y, size.z),
            ));
        }
    }

    if existing_obstacles.is_empty() {
        let obstacle_color = materials.add(StandardMaterial {
            base_color: Color::srgb(0.63, 0.32, 0.16),
            perceptual_roughness: 0.88,
            metallic: 0.0,
            ..default()
        });

        let obstacles = [
            (
                "Test World Obstacle 01",
                Vec3::new(4.0, 2.5, 4.0),
                Vec3::new(-12.0, 1.25, -8.0),
            ),
            (
                "Test World Obstacle 02",
                Vec3::new(3.0, 4.5, 3.0),
                Vec3::new(9.0, 2.25, -3.0),
            ),
            (
                "Test World Obstacle 03",
                Vec3::new(6.0, 2.0, 2.5),
                Vec3::new(-2.0, 1.0, 11.0),
            ),
            (
                "Test World Obstacle 04",
                Vec3::new(2.2, 3.2, 7.0),
                Vec3::new(15.0, 1.6, 12.0),
            ),
            (
                "Test World Obstacle 05",
                Vec3::new(5.0, 1.6, 5.0),
                Vec3::new(-16.0, 0.8, 14.0),
            ),
        ];

        for (name, size, position) in obstacles {
            commands.spawn((
                TestWorldObstacle,
                Name::new(name),
                Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
                MeshMaterial3d(obstacle_color.clone()),
                Transform::from_translation(position),
                RigidBody::Static,
                Collider::cuboid(size.x, size.y, size.z),
            ));
        }
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
