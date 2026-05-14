use bevy::app::App;
use bevy::prelude::Window;
use std::sync::{Mutex, OnceLock};

pub fn window_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&Window>();
    query.iter(world).count()
}

pub fn cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
