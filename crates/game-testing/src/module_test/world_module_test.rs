use bevy::prelude::*;

use crate::test_utils::window_count;
use world_module::spawn_test_world;

#[test]
fn world_world_systems_can_be_registered_without_spawning_windows() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, spawn_test_world);

    app.update();

    assert_eq!(window_count(&mut app), 0);
}
