pub mod client_window;
pub mod states;
mod systems;
pub mod ui;

use crate::systems::ClientSystemPlugin;
use crate::ui::ClientUiPlugin;
use avian3d::PhysicsPlugins;
use bevy::prelude::*;
use client_window::ClientWindowPlugin;
use game_shared::SharedLoadPlugin;

pub struct ClientPackedPlugin;

impl Plugin for ClientPackedPlugin {
    fn build(&self, app: &mut App) {
        // Add Crates Plugins
        app.add_plugins((
            SharedLoadPlugin,
            ClientSystemPlugin,
            ClientWindowPlugin,
            ClientUiPlugin,
        ));

        // Add Bevy Plugins
        app.add_plugins(PhysicsPlugins::default()); // Avian3D Physics
    }
}
