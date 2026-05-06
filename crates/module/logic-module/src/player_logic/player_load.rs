use bevy::prelude::*;
use game_shared::models::character::Character;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::http::account::AccountResource;
use game_shared::models::http::character::CharacterData;
use game_shared::models::EntityBase;

pub fn gen_player_from_response(
    response: Res<AccountResource>,
    mut character_group: ResMut<CharacterGroup>,
) {
    let Some(response) = response.0.as_ref() else {
        error!("Account Resource is None!");
        return;
    };

    let Some(active_team) = response.team.iter().find(|team| team.active) else {
        error!("No active team found in account response!");
        character_group.active_character = None;
        character_group.characters.clear();
        return;
    };

    character_group.characters = active_team
        .members
        .iter()
        .filter_map(|character_id| {
            response
                .characters
                .iter()
                .find(|character| character.id == *character_id)
                .map(character_from_data)
                .or_else(|| {
                    error!("Character with id {character_id} is missing in account response!");
                    None
                })
        })
        .collect();

    character_group.active_character = character_group
        .characters
        .iter()
        .find(|character| character.base.id == u64::from(active_team.current_use))
        .cloned();

    if character_group.active_character.is_none() {
        error!(
            "Current use character with id {} is missing in active team!",
            active_team.current_use
        );
    }

    info!(
        "Generated {} characters for active team '{}'",
        character_group.characters.len(),
        active_team.name
    );
}

fn character_from_data(character_data: &CharacterData) -> Character {
    Character {
        base: EntityBase {
            id: u64::from(character_data.id),
            localized_name: character_data.name.clone(),
            name: character_data.name.clone(),
            ..default()
        },
        ..default()
    }
}
