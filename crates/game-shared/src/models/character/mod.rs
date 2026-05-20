pub mod animation;
pub mod group;
pub mod world;

use crate::models::EntityBase;
pub use animation::CharacterAnimation;
use bevy::prelude::*;
use group::CharacterGroup;

pub struct CharacterApp;

impl Plugin for CharacterApp {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterGroup>();
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Character {
    pub base: EntityBase,
    pub attributes: CharacterAttributes,
}

#[derive(Component, Debug, Clone, Default)]
pub struct CharacterAttributes {
    pub mental_stamina: f64,

    // Pyro attributes for typed Characters
    pub pyro_power: f64,
    pub pyro_resistance: f64,
    // Water attributes for typed Characters
    pub water_power: f64,
    pub water_resistance: f64,
    // Cryo attributes for typed Characters
    pub cryo_power: f64,
    pub cryo_resistance: f64,
    // Electro attributes for typed Characters
    pub electro_power: f64,
    pub electro_resistance: f64,
    // Dark attributes for typed Characters
    pub dark_power: f64,
    pub dark_resistance: f64,
    // Holy attributes for typed Characters
    pub holy_power: f64,
    pub holy_resistance: f64,
    // Mental attributes for typed Characters
    pub mental_power: f64,
    pub mental_resistance: f64,
    // Physical attributes for typed Characters
    pub physical_power: f64,
    pub physical_resistance: f64,
}
