mod player_system;

use bevy::prelude::*;
use crate::systems::player_system::PlayerSystemComponent;

pub struct ClientSystemPlugin;

impl Plugin for ClientSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerSystemComponent);
    }
}