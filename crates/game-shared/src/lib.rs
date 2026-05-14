use crate::models::ModelsApp;
use bevy::prelude::*;

pub mod models;
pub mod utils;

pub struct SharedLoadPlugin;

impl Plugin for SharedLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ModelsApp);
    }
}
