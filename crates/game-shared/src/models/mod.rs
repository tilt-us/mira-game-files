pub mod character;
pub mod http;
pub mod player;

use bevy::prelude::*;

pub struct ModelsApp;

impl Plugin for ModelsApp {
    fn build(&self, app: &mut App) {
        app.add_plugins(http::HttpApp);
        app.add_plugins(character::CharacterApp);
    }
}

#[derive(Component, Debug, Clone)]
pub struct EntityBase {
    pub id: u64,
    pub localized_name: String,
    pub name: String,

    pub health: f64,
    pub defense: f64,
    pub super_armor: f64,
}

impl Default for EntityBase {
    fn default() -> Self {
        Self {
            id: 0,
            localized_name: String::new(),
            name: String::new(),
            health: 0.0,
            defense: 0.0,
            super_armor: 0.0,
        }
    }
}
