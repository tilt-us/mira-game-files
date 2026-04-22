use bevy::prelude::*;
use crate::states::{ClientState, LoadingState};

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::Loading(LoadingState::UiPreLoad)), load_ui_assets);
    }   
}

fn load_ui_assets(mut next_state: ResMut<NextState<ClientState>>) {
    info!("Loading UI assets complete.");
    next_state.set(ClientState::WindowVisible);
}