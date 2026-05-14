use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct TeamData {
    pub name: String,
    pub active: bool,
    pub members: Vec<u32>,
    pub leader: u32,
    pub current_use: u32,
}
