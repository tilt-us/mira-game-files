use bevy::prelude::*;

/// Represents metadata about the application's build information.
///
/// This structure is used to store important details about the application,
/// including its name, version, and the version of the Bevy engine it is built with.
/// It derives `Resource` for integration with Bevy's ECS system and provides
/// implementations of `Debug` and `Clone` for convenient debugging and cloning operations.
///
/// # Fields
/// - `name` (`String`): The name of the application.
/// - `version` (`String`): The version of the application.
/// - `bevy_version` (`String`): The version of the Bevy engine used for building the application.
#[derive(Resource, Debug, Clone)]
pub struct AppBuildInfo {
    pub name: String,
    pub version: String,
    pub bevy_version: String
}