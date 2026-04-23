use bevy::prelude::*;
use crate::states::{ClientState, LoadingState};

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::Loading(LoadingState::UiPreLoad)), load_ui_assets);
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
fn load_ui_assets(mut next_state: ResMut<NextState<ClientState>>) {
    info!("Loading UI assets complete.");
    next_state.set(ClientState::WindowVisible);
}