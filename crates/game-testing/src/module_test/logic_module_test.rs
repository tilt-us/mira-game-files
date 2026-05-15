use bevy::prelude::*;

use crate::test_utils::window_count;
use logic_module::camera_logic::camera_init::init_camera;

#[test]
fn logic_systems_can_be_registered_without_spawning_windows() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, init_camera);

    app.update();

    assert_eq!(window_count(&mut app), 0);
}
