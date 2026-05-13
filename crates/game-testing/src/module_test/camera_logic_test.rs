use bevy::prelude::*;

use logic_module::camera_logic::camera_init::init_camera;

fn camera_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&Camera3d>();
    query.iter(world).count()
}

#[test]
fn init_camera_spawns_one_camera_when_none_exists() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, init_camera);

    app.update();

    assert_eq!(camera_count(&mut app), 1);
}

#[test]
fn init_camera_does_not_spawn_duplicate_camera() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, init_camera);
    app.world_mut()
        .spawn((Camera3d::default(), Name::new("Existing Camera")));

    app.update();

    assert_eq!(camera_count(&mut app), 1);
}
