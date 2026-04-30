use bevy::app::App;
use bevy::prelude::Window;

pub fn window_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&Window>();
    query.iter(world).count()
}