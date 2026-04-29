use bevy::prelude::*;

use world_module::WorldModule;
use crate::test_utils::window_count;

#[test]
fn world_module_can_be_added_without_spawning_windows() {
    let mut app = App::new();
    app.add_plugins(WorldModule);

    app.update();

    assert_eq!(window_count(&mut app), 0);
}
