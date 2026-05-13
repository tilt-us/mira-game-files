use crate::states::ClientState;
use bevy::prelude::*;
use game_shared::models::http::account::AccountResource;
use logic_module::player_logic::player_load::gen_player_from_response;
use logic_module::player_logic::player_to_world::place_to_world;

pub struct PlayerSystemComponent;

impl Plugin for PlayerSystemComponent {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientState::WindowVisible),
            gen_player_from_response.run_if(resource_changed::<AccountResource>),
        );

        app.add_systems(
            OnEnter(ClientState::WindowVisible),
            place_to_world
                .after(gen_player_from_response)
                .run_if(resource_changed::<AccountResource>),
        );
    }
}
