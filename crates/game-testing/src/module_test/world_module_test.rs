use bevy::prelude::*;

use crate::test_utils::window_count;
use game_shared::models::world::{TestWorldFloor, TestWorldLight};
use world_module::spawn_test_world;

#[test]
fn world_world_systems_can_be_registered_without_spawning_windows() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, spawn_test_world);

    app.update();

    assert_eq!(window_count(&mut app), 0);
}

#[test]
fn spawn_test_world_creates_floor_and_light_once() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Update, spawn_test_world);

    app.update();
    app.update();

    let world = app.world_mut();
    let mut floor_query = world.query_filtered::<Entity, With<TestWorldFloor>>();
    let mut light_query = world.query_filtered::<Entity, With<TestWorldLight>>();
    assert_eq!(floor_query.iter(world).count(), 1);
    assert_eq!(light_query.iter(world).count(), 1);
}
