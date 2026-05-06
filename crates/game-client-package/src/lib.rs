pub mod client_window;
pub mod config;
pub mod states;
pub mod ui;
mod systems;

use avian3d::PhysicsPlugins;
use bevy::prelude::*;
use client_window::ClientWindowPlugin;
use game_shared::SharedLoadPlugin;
use logic_module::LogicModule;
use world_module::WorldModule;
use crate::systems::ClientSystemPlugin;
use crate::ui::ClientUiPlugin;

pub struct ClientPackedPlugin;

impl Plugin for ClientPackedPlugin {
    fn build(&self, app: &mut App) {
        // Add Crates Plugins
        app.add_plugins((
            SharedLoadPlugin,
            ClientSystemPlugin,
            ClientWindowPlugin,
            ClientUiPlugin
        ));

        // Add Module Plugins
        app.add_plugins((
            LogicModule,
            WorldModule
        ));

        // Add Bevy Plugins
        app.add_plugins(PhysicsPlugins::default()); // Avian3D Physics
    }
}
