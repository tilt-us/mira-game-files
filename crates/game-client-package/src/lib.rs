pub mod client_window;
pub mod states;
mod systems;
pub mod ui;
mod sys_info;

use crate::systems::ClientSystemPlugin;
use crate::ui::ClientUiPlugin;
use avian3d::PhysicsPlugins;
use bevy::prelude::*;
use bevy_extended_ui::{ExtendedUiConfiguration, ExtendedUiPlugin};
use client_window::ClientWindowPlugin;
use game_shared::SharedLoadPlugin;
use std::path::Path;
use crate::sys_info::DebugScreenUiPlugin;

pub struct ClientPackedPlugin;

impl Plugin for ClientPackedPlugin {
    fn build(&self, app: &mut App) {
        let mut ui_configuration = ExtendedUiConfiguration::default();
        ui_configuration.assets_path = shared_assets_subdir("ui");
        ui_configuration.themes_path = shared_assets_subdir("ui/css");
        app.insert_resource(ui_configuration);

        // Add Crates Plugins
        app.add_plugins((
            DebugScreenUiPlugin,
            SharedLoadPlugin,
            ClientSystemPlugin,
            ClientWindowPlugin,
            ClientUiPlugin,
        ));

        // Add Bevy Plugins
        app.add_plugins(PhysicsPlugins::default()); // Avian3D Physics

        app.add_plugins(ExtendedUiPlugin);
    }
}

fn shared_assets_subdir(subdir: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join(subdir)
        .to_string_lossy()
        .to_string()
}
