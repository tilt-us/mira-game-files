pub mod client_window;
pub mod config;
pub mod states;
pub mod ui;

use bevy::prelude::*;
use client_window::ClientWindowPlugin;
use crate::ui::ClientUiPlugin;

pub struct ClientPackedPlugin;

impl Plugin for ClientPackedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientWindowPlugin,
            ClientUiPlugin
        ));
    }
}
