use crate::states::ClientState;
use bevy::prelude::*;
use world_module::spawn_test_world;

/// Registers world setup systems required by the client runtime.
pub struct WorldSystemComponent;

impl Plugin for WorldSystemComponent {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::WindowVisible), spawn_test_world);
    }
}
