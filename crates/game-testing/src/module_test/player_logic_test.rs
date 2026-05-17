use crate::test_utils::cwd_lock;
use avian3d::prelude::{Collider, RigidBody};
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use game_shared::models::EntityBase;
use game_shared::models::character::Character;
use game_shared::models::character::group::CharacterGroup;
use game_shared::models::http::account::{AccountData, AccountResource, AccountResponse};
use game_shared::models::http::character::CharacterData;
use game_shared::models::http::team::TeamData;
use game_shared::models::player::{PartyCompanion, Player};
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
    let base_dir = character_definitions_dir();
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
    let path = character_definitions_dir().join(format!("{character_id}.json"));
    if path.exists() {
        fs::remove_file(path).expect("failed to remove character test json");
    }
}

fn character_definitions_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/entities/characters")
}

fn test_account_resource(character_id: u32, character_name: &str) -> AccountResource {
    test_account_resource_with_party(character_id, vec![(character_id, character_name)])
}

fn test_account_resource_with_party(
    current_use: u32,
    members: Vec<(u32, &str)>,
) -> AccountResource {
    let member_ids = members.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let characters = members
        .iter()
        .map(|(id, name)| CharacterData {
            id: *id,
            name: String::from(*name),
            level: 1,
        })
        .collect::<Vec<_>>();

    AccountResource(Some(AccountResponse {
        account: AccountData {
            id: 1,
            name: String::from("Test"),
            email: String::from("test@example.com"),
            birthday: String::from("01/01/2000"),
        },
        characters,
        teams: vec![TeamData {
            name: String::from("Test Team"),
            active: true,
            members: member_ids.clone(),
            leader: member_ids[0],
            current_use,
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

fn companion_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query::<&PartyCompanion>();
    query.iter(world).count()
}

fn companion_positions(app: &mut App) -> Vec<Vec3> {
    let world = app.world_mut();
    let mut query = world.query::<(&PartyCompanion, &Transform)>();
    query
        .iter(world)
        .map(|(_, transform)| transform.translation)
        .collect()
}

fn active_player_position(app: &mut App) -> Vec3 {
    let world = app.world_mut();
    let mut query = world.query::<(&Player, &Transform)>();
    query
        .iter(world)
        .next()
        .expect("expected a spawned player transform")
        .1
        .translation
}

fn create_app_with_player_systems(account_resource: AccountResource) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(AssetPlugin {
        file_path: project_assets_path(),
        ..default()
    });
    app.init_asset::<Scene>();
    app.insert_resource(account_resource);
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
    let mut app = create_app_with_player_systems(test_account_resource(character_id, "Ignara"));

    app.update();

    let character = active_player_character(&mut app);
    let player_name = active_player_name(&mut app);

    assert_eq!(player_count(&mut app), 1);
    assert_eq!(companion_count(&mut app), 0);
    assert_eq!(character.base.id, u64::from(character_id));
    assert_eq!(player_name, "Ignara");
}

#[test]
fn place_to_world_replaces_existing_player_entity() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let character_id = unique_character_id();
    let _cleanup =
        create_character_definition_for_id(character_id, "Lira", "missing_test_model.glb");
    let mut app = create_app_with_player_systems(test_account_resource(character_id, "Lira"));
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
    assert_eq!(companion_count(&mut app), 0);
    assert_eq!(character.base.id, u64::from(character_id));
}

#[test]
fn place_to_world_spawns_party_companions_around_player_and_avoids_fixed_objects() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let active_id = unique_character_id();
    let companion_a_id = unique_character_id();
    let companion_b_id = unique_character_id();
    let _cleanup_active =
        create_character_definition_for_id(active_id, "Astra", "missing_test_model.glb");
    let _cleanup_companion_a =
        create_character_definition_for_id(companion_a_id, "Boreal", "missing_test_model.glb");
    let _cleanup_companion_b =
        create_character_definition_for_id(companion_b_id, "Cyra", "missing_test_model.glb");

    let mut app = create_app_with_player_systems(test_account_resource_with_party(
        active_id,
        vec![
            (active_id, "Astra"),
            (companion_a_id, "Boreal"),
            (companion_b_id, "Cyra"),
        ],
    ));

    let obstacle_entity = app.world_mut().spawn((
        Name::new("Static Spawn Blocker"),
        Transform::from_xyz(4.0, 1.2, 0.0),
        RigidBody::Static,
        Collider::cuboid(2.0, 2.0, 2.0),
    ));
    let obstacle_entity = obstacle_entity.id();

    app.update();

    assert_eq!(player_count(&mut app), 1);
    assert_eq!(companion_count(&mut app), 2);

    let player_position = active_player_position(&mut app);
    let companion_positions = companion_positions(&mut app);

    let world = app.world();
    let obstacle_transform = world
        .entity(obstacle_entity)
        .get::<Transform>()
        .expect("expected obstacle transform");
    let obstacle_collider = world
        .entity(obstacle_entity)
        .get::<Collider>()
        .expect("expected obstacle collider");

    for companion_position in companion_positions {
        let distance_to_player = companion_position.distance(player_position);
        assert!(distance_to_player <= 10.0);

        let distance_to_obstacle = obstacle_collider.distance_to_point(
            obstacle_transform.translation,
            obstacle_transform.rotation,
            companion_position,
            true,
        );
        assert!(distance_to_obstacle >= 1.2);
    }
}
