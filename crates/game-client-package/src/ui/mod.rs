mod hud;

use crate::states::{ClientState, LoadingState};
use bevy::prelude::*;
use bevy_extended_ui::html::HtmlSource;
use bevy_extended_ui::io::HtmlAsset;
use bevy_extended_ui::registry::UiRegistry;
use game_shared::config::ClientConfigs;
use game_shared::utils::input_utils::detect_key_action;

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SystemInfoScreenState::default());
        app.add_systems(
            OnEnter(ClientState::Loading(LoadingState::UiPreLoad)),
            load_ui_assets,
        );
        app.add_systems(
            Update,
            toggle_system_info_screen.run_if(in_state(ClientState::WindowVisible)),
        );
    }
}

#[derive(Resource, Default)]
struct SystemInfoScreenState {
    visible: bool,
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
                  mut system_info_state: ResMut<SystemInfoScreenState>,
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

    system_info_state.visible = false;
    ui_registry.use_uis(vec!["player_hud".to_string()]);

    info!("Loading UI assets complete.");
    next_state.set(ClientState::WindowVisible);
}

fn toggle_system_info_screen(
    keyboard: Res<ButtonInput<KeyCode>>,
    client_configs: Res<ClientConfigs>,
    mut system_info_state: ResMut<SystemInfoScreenState>,
    mut ui_registry: ResMut<UiRegistry>,
) {
    if !detect_key_action(
        client_configs.config_input.system_info_screen().to_string(),
        &keyboard,
    ) {
        return;
    }

    system_info_state.visible = !system_info_state.visible;
    if system_info_state.visible {
        ui_registry.use_uis(vec![
            "player_hud".to_string(),
            "debug_screen".to_string(),
        ]);
    } else {
        ui_registry.use_uis(vec!["player_hud".to_string()]);
    }
}
