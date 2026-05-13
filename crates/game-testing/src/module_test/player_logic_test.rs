use crate::test_utils::cwd_lock;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::character::Character;
use game_shared::models::http::account::{AccountData, AccountResource, AccountResponse};
use game_shared::models::http::character::CharacterData;
use game_shared::models::http::team::TeamData;
use game_shared::models::player::Player;
use game_shared::models::EntityBase;
use logic_module::player_logic::player_load::gen_player_from_response;
use logic_module::player_logic::player_to_world::place_to_world;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

fn project_assets_path() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .to_string_lossy()
        .to_string()
}

fn unique_character_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(1);
    let process_bits = (std::process::id() & 0x7FFF) << 16;
    let sequence_bits = NEXT_ID.fetch_add(1, Ordering::Relaxed) & 0xFFFF;
    process_bits | sequence_bits
}

struct CharacterDefinitionCleanup {
    character_id: u32,
}

impl Drop for CharacterDefinitionCleanup {
    fn drop(&mut self) {
        remove_character_definition_for_id(self.character_id);
    }
}

fn create_character_definition_for_id(
    character_id: u32,
    display_name: &str,
    model_name: &str,
) -> CharacterDefinitionCleanup {
    let base_dir = Path::new("assets/entities/characters");
    fs::create_dir_all(&base_dir).expect("failed to create local character test directory");

    let path = base_dir.join(format!("{character_id}.json"));
    let json = format!(
        "{{\n  \"localized_name\": \"{}\",\n  \"display_name\": \"{}\",\n  \"full_name\": \"{}\",\n  \"model_name\": \"{}\"\n}}",
        display_name.to_lowercase(),
        display_name,
        display_name,
        model_name
    );

    fs::write(path, json).expect("failed to create character test json");
    CharacterDefinitionCleanup { character_id }
}

fn remove_character_definition_for_id(character_id: u32) {
    let path = Path::new("assets/entities/characters").join(format!("{character_id}.json"));
    if path.exists() {
        fs::remove_file(path).expect("failed to remove character test json");
    }
}

fn test_account_resource(character_id: u32, character_name: &str) -> AccountResource {
    AccountResource(Some(AccountResponse {
        account: AccountData {
            id: 1,
            name: String::from("Test"),
            email: String::from("test@example.com"),
            birthday: String::from("01/01/2000"),
        },
        characters: vec![CharacterData {
            id: character_id,
            name: String::from(character_name),
            level: 1,
        }],
        team: vec![TeamData {
            name: String::from("Test Team"),
            active: true,
            members: vec![character_id],
            leader: character_id,
            current_use: character_id,
        }],
    }))
}

fn player_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&Player>();
    query.iter(world).count()
}

fn active_player_character(app: &mut App) -> Character {
    let world = app.world_mut();
    let mut query = world.query::<(&Player, &Character)>();
    query
        .iter(world)
        .next()
        .expect("expected a spawned player character")
        .1
        .clone()
}

fn active_player_name(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query::<(&Player, &Name)>();
    query
        .iter(world)
        .next()
        .expect("expected a spawned player name")
        .1
        .to_string()
}

fn create_app_with_player_systems(character_id: u32, character_name: &str) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<Scene>();
    app.insert_resource(test_account_resource(character_id, character_name));
    app.init_resource::<CharacterGroup>();
    app.add_systems(
        Update,
        (
            gen_player_from_response,
            place_to_world.after(gen_player_from_response),
        ),
    );
    app
}

#[test]
fn place_to_world_spawns_current_use_character() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let character_id = unique_character_id();
    let _cleanup =
        create_character_definition_for_id(character_id, "Ignara", "missing_test_model.glb");
    let mut app = create_app_with_player_systems(character_id, "Ignara");

    app.update();

    let character = active_player_character(&mut app);
    let player_name = active_player_name(&mut app);

    assert_eq!(player_count(&mut app), 1);
    assert_eq!(character.base.id, u64::from(character_id));
    assert_eq!(player_name, "Ignara");
}

#[test]
fn place_to_world_replaces_existing_player_entity() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let character_id = unique_character_id();
    let _cleanup =
        create_character_definition_for_id(character_id, "Lira", "missing_test_model.glb");
    let mut app = create_app_with_player_systems(character_id, "Lira");
    app.world_mut().spawn((
        Player,
        Character {
            base: EntityBase {
                id: 9999,
                localized_name: String::from("old"),
                name: String::from("old"),
                ..default()
            },
            ..default()
        },
        Name::new("Old Player"),
    ));

    app.update();

    let character = active_player_character(&mut app);

    assert_eq!(player_count(&mut app), 1);
    assert_eq!(character.base.id, u64::from(character_id));
}
