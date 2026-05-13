use bevy::prelude::*;
use game_shared::models::character::Character;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::http::account::AccountResource;
use game_shared::models::http::character::CharacterData;
use game_shared::models::EntityBase;

/// Generates a player's character data based on the provided account response and updates the given character group structure.
///
/// # Parameters
/// - `response`: A reference to an `AccountResource` that contains the account's data, including teams and characters.
/// - `character_group`: A mutable reference to a `CharacterGroup` where the generated characters and active character will be stored.
///
/// # Behavior
/// 1. If the `response` is `None`, an error is logged, and the function exits early.
/// 2. Finds the active team in the account response. If no active team is found:
///     - An error is logged.
///     - The `character_group`'s `active_character` is set to `None`.
///     - Any previously stored `characters` in `character_group` are cleared.
/// 3. Iterates through the active team's member IDs, attempts to match the IDs with characters in the account response,
///    and converts the matching character data into a structure using the `character_from_data` function:
///     - If a character ID is missing in the account response, an error is logged.
///     - Only valid characters are added to the `character_group`.
/// 4. Sets the `active_character` in the `character_group` to the character whose ID matches the `current_use` field of the active team:
///     - If the `current_use` character is not found, an error is logged.
/// 5. Logs a confirmation message indicating the number of characters generated and the name of the active team.
///
/// # Logging
/// - Logs errors if the account resource or required data (active team, character data) is missing or invalid.
/// - Logs information about the number of characters generated and the active team's name in the end.
///
/// # Example
/// ```ignore
/// let account_response = Res(AccountResource::new(...)); // Assuming valid data initialization.
/// let mut character_group = ResMut(CharacterGroup::new());
///
/// gen_player_from_response(account_response, character_group);
///
/// assert!(character_group.active_character.is_some());
/// assert!(!character_group.characters.is_empty());
/// ```
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

/// Converts a `CharacterData` object into a `Character` object.
///
/// # Parameters
/// - `character_data`: A reference to the `CharacterData` instance containing
///   the necessary information to create a `Character`.
///
/// # Returns
/// A `Character` object initialized using the data from the supplied
/// `character_data`.
///
/// The function populates the following values:
/// - `id` is converted from `u32` to `u64` using the `id` property of `character_data`.
/// - `localized_name` is a clone of the `name` field of `character_data`.
/// - `name` is also a clone of the `name` field of `character_data`.
///
/// Default values are applied to fields of both `EntityBase` and `Character` where
/// data is not explicitly provided. Ensure that the `default()` method is properly
/// implemented for these types.
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
