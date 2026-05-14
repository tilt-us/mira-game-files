use crate::models::character::Character;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct CharacterGroup {
    pub active_character: Option<Character>,
    pub characters: Vec<Character>,
}
