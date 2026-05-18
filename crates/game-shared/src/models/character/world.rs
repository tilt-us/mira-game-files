use super::CharacterAnimation;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Represents world-specific character metadata loaded from character definition files.
///
/// These fields are intentionally decoupled from account payloads. Account responses
/// provide ownership and progression data (for example ID and level), while this
/// structure provides static character presentation, model mapping, and animation
/// mapping data.
///
/// # File Format
/// A character definition JSON must provide:
/// - `localized_name`: internal or localized identifier text.
/// - `display_name`: player-facing short name.
/// - `full_name`: canonical character name.
/// - `model_name`: model file name (for example, `lira.glb`).
/// - `animations`: animation mappings with clip keys and GLB animation indices.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CharacterWorldData {
    /// Internal or localized identifier text.
    pub localized_name: String,
    /// Player-facing short name.
    pub display_name: String,
    /// Canonical character name.
    pub full_name: String,
    /// GLB file name used for scene and animation loading.
    pub model_name: String,
    /// Mapping from logical animation keys to GLB animation indices.
    pub animations: Vec<CharacterAnimation>,
}

impl CharacterWorldData {
    /// Builds a [`CharacterWorldData`] instance by loading the character definition
    /// file for the provided character ID.
    ///
    /// # Parameters
    /// - `character_id`: The character ID. The loader resolves the file path as
    ///   `assets/entities/characters/{character_id}.json`.
    ///
    /// # Returns
    /// - `Ok(CharacterWorldData)` when the file exists and contains valid JSON
    ///   including animation mapping entries.
    /// - `Err(String)` with context if the file cannot be read or parsed.
    pub fn build_from_id(character_id: u64) -> Result<Self, String> {
        let path = character_definition_path(character_id);
        Self::build_from_json(path)
    }

    /// Builds a [`CharacterWorldData`] instance from a JSON file path.
    ///
    /// # Parameters
    /// - `path`: The path of the JSON file to load.
    ///
    /// # Returns
    /// - `Ok(CharacterWorldData)` if the file is readable and valid JSON including
    ///   animation mappings.
    /// - `Err(String)` containing a descriptive error message otherwise.
    pub fn build_from_json<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path).map_err(|error| {
            format!(
                "Failed to read character definition file '{}': {error}",
                path.display()
            )
        })?;

        Self::from_json_str(&json).map_err(|error| {
            format!(
                "Failed to parse character definition file '{}': {error}",
                path.display()
            )
        })
    }

    /// Builds a [`CharacterWorldData`] instance from a JSON string.
    ///
    /// # Parameters
    /// - `json`: A JSON string in character definition format.
    ///
    /// # Returns
    /// - `Ok(CharacterWorldData)` if the JSON can be deserialized.
    /// - `Err(serde_json::Error)` if deserialization fails.
    fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Resolves the expected character definition file path for a character ID.
///
/// # Parameters
/// - `character_id`: The character ID to resolve.
///
/// # Returns
/// A [`PathBuf`] pointing to `assets/entities/characters/{character_id}.json`.
fn character_definition_path(character_id: u64) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join("entities")
        .join("characters")
        .join(format!("{character_id}.json"))
}
