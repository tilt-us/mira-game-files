use bevy::prelude::*;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::http::account::{AccountData, AccountResource, AccountResponse};
use game_shared::models::http::character::CharacterData;
use game_shared::models::http::team::TeamData;
use logic_module::player_logic::player_load::gen_player_from_response;

fn account_with_team(active: bool, current_use: u32, members: Vec<u32>) -> AccountResource {
    AccountResource(Some(AccountResponse {
        account: AccountData {
            id: 1,
            name: String::from("Tester"),
            email: String::from("tester@example.com"),
            birthday: String::from("01/01/2000"),
        },
        characters: vec![
            CharacterData {
                id: 11,
                name: String::from("Alpha"),
                level: 1,
            },
            CharacterData {
                id: 22,
                name: String::from("Beta"),
                level: 1,
            },
        ],
        teams: vec![TeamData {
            name: String::from("Main Team"),
            active,
            members,
            leader: 11,
            current_use,
        }],
    }))
}

#[test]
fn gen_player_from_response_builds_characters_for_active_team() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(account_with_team(true, 11, vec![11, 22]));
    app.insert_resource(CharacterGroup::default());
    app.add_systems(Update, gen_player_from_response);

    app.update();

    let group = app.world().resource::<CharacterGroup>();
    assert_eq!(group.characters.len(), 2);
    let active = group
        .active_character
        .as_ref()
        .expect("expected active character");
    assert_eq!(active.base.id, 11);
    assert_eq!(active.base.name, "Alpha");
}

#[test]
fn gen_player_from_response_clears_group_when_no_active_team() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(account_with_team(false, 11, vec![11, 22]));
    app.insert_resource(CharacterGroup {
        active_character: None,
        characters: vec![Default::default()],
    });
    app.add_systems(Update, gen_player_from_response);

    app.update();

    let group = app.world().resource::<CharacterGroup>();
    assert!(group.characters.is_empty());
    assert!(group.active_character.is_none());
}

#[test]
fn gen_player_from_response_handles_missing_account_without_mutation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(AccountResource(None));
    app.insert_resource(CharacterGroup::default());
    app.add_systems(Update, gen_player_from_response);

    app.update();

    let group = app.world().resource::<CharacterGroup>();
    assert!(group.characters.is_empty());
    assert!(group.active_character.is_none());
}

#[test]
fn gen_player_from_response_skips_missing_member_and_missing_current_use() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(account_with_team(true, 999, vec![11, 999]));
    app.insert_resource(CharacterGroup::default());
    app.add_systems(Update, gen_player_from_response);

    app.update();

    let group = app.world().resource::<CharacterGroup>();
    assert_eq!(group.characters.len(), 1);
    assert_eq!(group.characters[0].base.id, 11);
    assert!(group.active_character.is_none());
}
