use crate::models::http::character::CharacterData;
use crate::models::http::team::TeamData;
use bevy::prelude::Resource;
use serde::Deserialize;
use std::path::Path;

#[derive(Resource, Clone)]
pub struct AccountResource(pub Option<AccountResponse>);

#[derive(Deserialize, Debug, Clone, Default)]
pub struct AccountResponse {
    pub account: AccountData,
    pub characters: Vec<CharacterData>,
    pub teams: Vec<TeamData>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct AccountData {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub birthday: String,
}

impl Default for AccountResource {
    fn default() -> Self {
        Self(AccountResponse::get_default_account().into())
    }
}

impl AccountResponse {
    pub fn build_from_json<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let json = std::fs::read_to_string(path).unwrap_or_else(|error| {
            panic!(
                "Failed to read account json file '{}': {error}",
                path.display()
            )
        });

        Self::from_json_str(&json)
    }

    pub fn get_default_account() -> Self {
        Self::from_json_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../http_examples/fake_account_from_server.json"
        )))
    }

    fn from_json_str(json: &str) -> Self {
        serde_json::from_str(json).expect("Failed to build AccountResponse from json")
    }
}
