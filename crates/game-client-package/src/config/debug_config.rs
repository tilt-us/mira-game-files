use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct AppBuildInfo {
    pub name: String,
    pub version: String,
    pub bevy_version: String
}