use bevy::prelude::*;

/// Marker component for the generated physics and render test floor.
#[derive(Component, Debug, Clone)]
pub struct TestWorldFloor;

/// Marker component for the generated directional daylight entity.
#[derive(Component, Debug, Clone)]
pub struct TestWorldLight;
