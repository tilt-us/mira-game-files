use bevy::prelude::*;

/// Marker component for the generated physics and render test floor.
#[derive(Component, Debug, Clone)]
pub struct TestWorldFloor;

/// Marker component for the generated directional daylight entity.
#[derive(Component, Debug, Clone)]
pub struct TestWorldLight;

/// Marker component for generated room boundary walls in the test world.
#[derive(Component, Debug, Clone)]
pub struct TestWorldWall;

/// Marker component for generated static obstacles in the test world.
#[derive(Component, Debug, Clone)]
pub struct TestWorldObstacle;
