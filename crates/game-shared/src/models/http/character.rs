use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CharacterData {
    pub id: u32,
    pub name: String,
    pub level: u32,
}
