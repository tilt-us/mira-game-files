mod hud;

use crate::states::{ClientState, LoadingState};
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientState::Loading(LoadingState::UiPreLoad)),
            load_ui_assets,
        );
    }
}

/// Loads the UI assets and transitions the application state.
///
/// This function logs a message indicating that the UI assets have been
/// successfully loaded. After the assets are loaded, it updates the application's
/// state to [ClientState::WindowVisible].
///
/// # Arguments
///
/// * `next_state` - A mutable resource of type `ResMut<NextState<ClientState>>`
///   that allows transitioning to the next client state.
fn load_ui_assets(mut next_state: ResMut<NextState<ClientState>>,
                  mut ui_registry: ResMut<UiRegistry>,
                  asset_server: Res<AssetServer>
) {
    // HUD
    let hud_handle: Handle<HtmlAsset> = asset_server.load("ui/player_hud.html");
    ui_registry.add(
        "player_hud".to_string(),
        HtmlSource::from_handle(hud_handle)
    );
    // DEBUG
    let debug_handle: Handle<HtmlAsset> = asset_server.load("ui/debug.html");
    ui_registry.add(
        "debug_screen".to_string(), 
        HtmlSource::from_handle(debug_handle)
    );

    ui_registry.use_uis(vec![
        "player_hud".to_string(),
        "debug_screen".to_string()
    ]);

    info!("Loading UI assets complete.");
    next_state.set(ClientState::WindowVisible);
}
