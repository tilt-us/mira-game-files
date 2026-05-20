use crate::states::ClientState;
use bevy::prelude::*;
use game_shared::config::ClientConfigs;
use game_shared::models::http::account::AccountResource;
use game_shared::models::player::{PlayerMovementInputConfig, PlayerPartyInputConfig};
use logic_module::camera_logic::camera_follow::follow_player_orbit_camera;
use logic_module::player_logic::character_animation::{
    setup_character_animation, update_character_animation_state,
};
use logic_module::player_logic::character_occlusion::{
    apply_character_occlusion_material_alpha, attach_occlusion_material_owners,
    bind_occlusion_material_instances, update_character_occlusion_targets,
};
use logic_module::player_logic::player_load::gen_player_from_response;
use logic_module::player_logic::player_movement::{
    party_companion_follow, player_movement_detect, swap_active_party_character,
    update_player_grounded,
};
use logic_module::player_logic::player_to_world::place_to_world;
use world_module::spawn_test_world;

/// Registers player generation, world placement, and movement systems.
pub struct PlayerSystemComponent;

impl Plugin for PlayerSystemComponent {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientState::WindowVisible),
            sync_input_config_from_client,
        );

        app.add_systems(
            OnEnter(ClientState::WindowVisible),
            (
                gen_player_from_response,
                place_to_world
                    .after(gen_player_from_response)
                    .after(spawn_test_world),
            ),
        );

        app.add_systems(
            Update,
            sync_input_config_from_client
                .run_if(in_state(ClientState::WindowVisible))
                .run_if(resource_changed::<ClientConfigs>),
        );

        app.add_systems(
            Update,
            gen_player_from_response
                .run_if(in_state(ClientState::WindowVisible))
                .run_if(resource_changed::<AccountResource>),
        );

        app.add_systems(
            Update,
            place_to_world
                .after(gen_player_from_response)
                .run_if(in_state(ClientState::WindowVisible))
                .run_if(resource_changed::<AccountResource>),
        );

        app.add_systems(
            Update,
            setup_character_animation.run_if(in_state(ClientState::WindowVisible)),
        );

        app.add_systems(
            Update,
            (
                swap_active_party_character,
                update_player_grounded,
                player_movement_detect,
                party_companion_follow,
                update_character_animation_state,
            )
                .chain()
                .after(setup_character_animation)
                .run_if(in_state(ClientState::WindowVisible))
                .run_if(resource_exists::<PlayerMovementInputConfig>),
        );

        app.add_systems(
            Update,
            (
                attach_occlusion_material_owners,
                bind_occlusion_material_instances.after(attach_occlusion_material_owners),
                update_character_occlusion_targets
                    .after(party_companion_follow)
                    .after(follow_player_orbit_camera),
                apply_character_occlusion_material_alpha
                    .after(update_character_occlusion_targets)
                    .after(bind_occlusion_material_instances),
            )
                .run_if(in_state(ClientState::WindowVisible)),
        );
    }
}

/// Synchronizes the input configuration from the client and updates the game's input settings.
///
/// # Parameters
/// - `commands`: A mutable instance of `Commands` that allows for the manipulation of game resources and entities.
/// - `client_configs`: A resource containing the client's configuration settings, specifically the input mappings.
///
/// This function retrieves the input configuration from the provided `ClientConfigs` resource and maps the client's
/// input settings to the game's internal `PlayerMovementInputConfig` resource. The mapping includes keys for
/// forward movement, backward movement, strafing left and right, jumping, sprinting, and sneaking. These settings
/// are then stored as a new resource, overriding any existing input configuration for the player.
///
/// # Purpose
/// Ensures that the game's input configuration reflects the client's settings, allowing for customized controls
/// by the player.
fn sync_input_config_from_client(mut commands: Commands, client_configs: Res<ClientConfigs>) {
    let config = &client_configs.config_input;
    commands.insert_resource(PlayerMovementInputConfig {
        movement_forward: config.movement_forward().to_string(),
        movement_backward: config.movement_backward().to_string(),
        movement_left: config.movement_left().to_string(),
        movement_right: config.movement_right().to_string(),
        movement_jump: config.movement_jump().to_string(),
        movement_sprint: config.movement_sprint().to_string(),
        movement_sneak: config.movement_sneak().to_string(),
    });

    commands.insert_resource(PlayerPartyInputConfig {
        party_slot_01: config.party_slot_01().to_string(),
        party_slot_02: config.party_slot_02().to_string(),
        party_slot_03: config.party_slot_03().to_string(),
        party_slot_04: config.party_slot_04().to_string(),
        party_next_slot: config.party_next_slot().to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_input_config_from_client_inserts_movement_and_party_bindings() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ClientConfigs::default());
        app.add_systems(Update, sync_input_config_from_client);

        app.update();

        let movement = app.world().resource::<PlayerMovementInputConfig>();
        assert_eq!(movement.movement_forward, "W");
        assert_eq!(movement.movement_backward, "S");
        assert_eq!(movement.movement_left, "A");
        assert_eq!(movement.movement_right, "D");
        assert_eq!(movement.movement_jump, "Space");
        assert_eq!(movement.movement_sprint, "ShiftLeft");
        assert_eq!(movement.movement_sneak, "CtrlLeft");

        let party = app.world().resource::<PlayerPartyInputConfig>();
        assert_eq!(party.party_slot_01, "1");
        assert_eq!(party.party_slot_02, "2");
        assert_eq!(party.party_slot_03, "3");
        assert_eq!(party.party_slot_04, "4");
        assert_eq!(party.party_next_slot, "Q");
    }
}
