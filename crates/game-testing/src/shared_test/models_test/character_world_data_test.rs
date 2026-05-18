use crate::test_utils::cwd_lock;
use game_shared::models::character::world::CharacterWorldData;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

fn unique_character_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let process_bits = (u64::from(std::process::id()) & 0xFFFF_FFFF) << 24;
    let sequence_bits = NEXT_ID.fetch_add(1, Ordering::Relaxed) & 0x00FF_FFFF;
    process_bits | sequence_bits
}

struct CharacterDefinitionCleanup {
    character_id: u64,
}

impl Drop for CharacterDefinitionCleanup {
    fn drop(&mut self) {
        remove_character_definition_for_id(self.character_id);
    }
}

fn create_character_definition_for_id(character_id: u64) -> CharacterDefinitionCleanup {
    let base_dir = character_definitions_dir();
    fs::create_dir_all(&base_dir).expect("failed to create local character test directory");

    let path = base_dir.join(format!("{character_id}.json"));
    let json = r#"{
  "localized_name": "test-localized",
  "display_name": "Test Display",
  "full_name": "Test Full",
  "model_name": "test.glb",
  "animations": [
    {
      "key": "idle",
      "index": 0
    },
    {
      "key": "walk",
      "index": 1
    }
  ]
}"#;

    fs::write(path, json).expect("failed to create character test json");
    CharacterDefinitionCleanup { character_id }
}

fn remove_character_definition_for_id(character_id: u64) {
    let path = character_definitions_dir().join(format!("{character_id}.json"));
    if path.exists() {
        fs::remove_file(path).expect("failed to remove character test json");
    }
}

fn character_definitions_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/entities/characters")
}

#[test]
fn build_from_json_reads_character_definition() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let definition_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/entities/characters/6606.json");
    let world_data = CharacterWorldData::build_from_json(definition_path)
        .expect("expected character definition to be readable");

    assert_eq!(world_data.localized_name, "lira");
    assert_eq!(world_data.display_name, "Lira");
    assert_eq!(world_data.full_name, "Lira");
    assert_eq!(world_data.model_name, "lira.glb");
    assert_eq!(world_data.animations.len(), 5);
    assert_eq!(world_data.animations[0].key, "idle");
    assert_eq!(world_data.animations[0].index, 1);
}

#[test]
fn build_from_id_reads_existing_character_definition() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let character_id = unique_character_id();
    let _cleanup = create_character_definition_for_id(character_id);

    let world_data = CharacterWorldData::build_from_id(character_id)
        .expect("expected character definition by id to be readable");

    assert_eq!(world_data.localized_name, "test-localized");
    assert_eq!(world_data.display_name, "Test Display");
    assert_eq!(world_data.full_name, "Test Full");
    assert_eq!(world_data.model_name, "test.glb");
    assert_eq!(world_data.animations.len(), 2);
    assert_eq!(world_data.animations[1].key, "walk");
    assert_eq!(world_data.animations[1].index, 1);
}

#[test]
fn build_from_id_returns_error_for_missing_definition() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let result = CharacterWorldData::build_from_id(99_999_999);

    assert!(result.is_err());
}

#[test]
fn build_from_id_works_independent_from_current_working_directory() {
    let _guard = cwd_lock().lock().expect("failed to lock current dir");
    let original_dir = env::current_dir().expect("failed to read current dir");
    let app_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/game-client");

    env::set_current_dir(&app_dir).expect("failed to switch to app directory");

    let result = CharacterWorldData::build_from_id(6606);

    env::set_current_dir(original_dir).expect("failed to restore current dir");

    let world_data = result.expect("expected character definition by id to be readable");
    assert_eq!(world_data.display_name, "Lira");
}
