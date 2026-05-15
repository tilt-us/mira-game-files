use bevy::asset::AssetPlugin;
use bevy::log::{DEFAULT_FILTER, LogPlugin};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings, WgpuSettingsPriority};
use bevy::window::ExitCondition;
use game_client_package::ClientPackedPlugin;
use game_client_package::states::{ClientState, LoadingState};
use game_shared::config::ClientConfigs;
use game_shared::config::debug_config::AppBuildInfo;
use std::path::Path;

/// The `main` function serves as the entry point for the application.
///
/// - Initializes the client configuration by creating an instance of `ClientConfigs`.
/// - Creates a new instance of the `App` structure, which is responsible for managing the lifecycle of the application.
/// - Calls the `boot_bevy_app` function, passing a mutable reference to the `App` instance and a reference to the client configuration.
///
/// This setup prepares and configures the application for execution, based on the provided configurations and Bevy engine initialization.
fn main() {
    let client_config = ClientConfigs::new();
    let mut app = App::new();
    boot_bevy_app(&mut app, &client_config);
}

/// Initializes and boots the Bevy application with the specified configuration and plugins.
///
/// # Parameters
/// - `app`: A mutable reference to the Bevy `App` instance to be configured and run.
/// - `config`: A reference to the `ClientConfigs` structure containing configuration details for the client.
///
/// # Functionality
/// 1. Sets up build information for the application, such as the app name, version, and Bevy engine version.
/// 2. Inserts the client configuration (`config`) and build information as shared application resources.
/// 3. Configures the application plugins:
///    - **DefaultPlugins** are initialized with custom modifications:
///        * `WindowPlugin`: Configured to prevent the app from exiting when a window is closed (`ExitCondition::DontExit`).
///        * `RenderPlugin`: Automatically selects a GPU backend for rendering, based on the provided `ClientConfigs`.
///        * `ImagePlugin`: Configured to use the nearest neighbor scaling mode for rendering 2D textures.
/// 4. Adds the `ClientPackedPlugin` to the app, which contains client-specific systems and logic.
/// 5. Initializes the `ClientState` state machine.
/// 6. Adds a startup system that transitions the client state to `Loading(LoadingState::UiPreLoad)`.
/// 7. Runs the application.
///
/// # Notes
/// - The `app` is expected to be a mutable reference to a pre-configured `App` object.
/// - The `ClientConfigs` struct is cloned into the application, allowing centralized access to configuration data.
/// - The function is tightly integrated with the Bevy framework (version `0.18.1`) and assumes the presence of specific plugins and extensions.
///
/// # Dependencies
/// - **Bevy** 0.18.1
/// - Custom application structures:
///   - `ClientConfigs`
///   - `ClientPackedPlugin`
///   - `ClientState`, including the state variant `Loading(LoadingState::UiPreLoad)`
fn boot_bevy_app(app: &mut App, config: &ClientConfigs) {
    let app_build_info = AppBuildInfo {
        name: "Mira: Fallen Gates".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        bevy_version: "0.18.1".to_string(),
    };

    app.insert_resource(config.clone());
    app.insert_resource(app_build_info);

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: project_assets_path(),
                ..default()
            })
            .set(LogPlugin {
                filter: format!("{DEFAULT_FILTER}bevy_gltf::loader=error,"),
                ..default()
            })
            .set(WindowPlugin {
                exit_condition: ExitCondition::DontExit,
                primary_window: None,
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(select_gpu_backend(config.clone())),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    app.add_plugins(ClientPackedPlugin);

    app.init_state::<ClientState>();

    app.add_systems(Startup, |mut next_state: ResMut<NextState<ClientState>>| {
        next_state.set(ClientState::Loading(LoadingState::UiPreLoad));
    });

    app.run();
}

/// Resolves the absolute path to the shared project asset folder.
///
/// # Returns
/// A UTF-8 string path pointing to `<workspace-root>/assets`.
///
/// # Notes
/// The executable crate is located in `apps/game-client`. This function maps
/// that location to the workspace-level asset directory so all modules can load
/// assets from a single source of truth.
fn project_assets_path() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .to_string_lossy()
        .to_string()
}

/// Selects the appropriate GPU backend based on client configuration.
///
/// # Arguments
/// * `client_configs` - A `ClientConfigs` instance containing the graphics configuration
/// for determining the desired GPU backend.
///
/// # Returns
/// * `WgpuSettings` - A structure containing the selected GPU backend (`backends`)
/// and other settings initialized with their default values, except the priority,
/// which is set to `WgpuSettingsPriority::Functionality`.
///
/// # Notes
/// * Outputs a warning for unknown backends.
/// * Assumes `default()` initializes other `WgpuSettings` fields appropriately.
///
/// # Dependencies
/// * Requires `ClientConfigs`, which must include a field `config_graphics`
/// containing a `graphic_backend` string.
///
/// # Error Handling
/// * Unknown backend strings are logged, but no errors are returned.
fn select_gpu_backend(client_configs: ClientConfigs) -> WgpuSettings {
    let backend = match client_configs
        .config_graphics
        .graphic_backend
        .to_ascii_lowercase()
        .as_str()
    {
        "auto" | "primary" => Some(Backends::PRIMARY),
        "vulkan" | "vlk" => Some(Backends::VULKAN),
        "dx12" | "directx12" | "d12" | "direct12" | "dx" => Some(Backends::DX12),
        "metal" => Some(Backends::METAL),
        "opengl" | "gl" => Some(Backends::GL),
        other => {
            eprintln!("Unknown backend '{}', falling back to PRIMARY", other);
            Some(Backends::PRIMARY)
        }
    };

    WgpuSettings {
        backends: backend,
        priority: WgpuSettingsPriority::Functionality,
        ..default()
    }
}
