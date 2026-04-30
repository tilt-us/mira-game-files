pub mod client_window;
pub mod config;
pub mod states;
pub mod ui;

use avian3d::PhysicsPlugins;
use bevy::prelude::*;
use client_window::ClientWindowPlugin;
use logic_module::LogicModule;
use world_module::WorldModule;
use crate::ui::ClientUiPlugin;

pub struct ClientPackedPlugin;

impl Plugin for ClientPackedPlugin {
    fn build(&self, app: &mut App) {
        // Add Crates Plugins
        app.add_plugins((
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
