use crate::states::ClientState;
use bevy::prelude::*;
use bevy::window::{
    MonitorSelection, PresentMode, PrimaryWindow, Window, WindowMode, WindowResolution,
};
use game_shared::config::ClientConfigs;

pub struct ClientWindowPlugin;

impl Plugin for ClientWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ClientState::WindowVisible),
            create_or_show_primary_window,
        );
    }
}

/// Creates or shows the primary application window.
///
/// This function checks if a primary window exists. If it does not, it spawns a new primary window
/// with properties configured based on the provided settings within `ClientConfigs`. If a primary
/// window already exists, it ensures the window is set to be visible.
///
/// # Parameters
/// - `commands`: A mutable reference to the `Commands` object, used to spawn new entities.
/// - `config`: A reference to the `ClientConfigs` resource, containing configuration details
///   like fullscreen mode, window dimensions, and other graphics-specific options.
/// - `windows`: A query that retrieves windows tagged with the `PrimaryWindow` component.
///
/// # Notes
/// - The window visibility is always enforced for existing primary windows.
/// - The `vsync_mode` function is expected to compute the appropriate present mode based on the
///   configuration.
fn create_or_show_primary_window(
    mut commands: Commands,
    config: Res<ClientConfigs>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if windows.is_empty() {
        commands.spawn((
            Window {
                title: String::from("Mira: Fallen Gates"),
                mode: if config.config_graphics.fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                } else {
                    WindowMode::Windowed
                },
                resolution: WindowResolution::new(
                    config.config_graphics.window_width,
                    config.config_graphics.window_height,
                ),
                present_mode: vsync_mode(&config),
                visible: true,
                ..default()
            },
            PrimaryWindow,
        ));

        info!("Primary window created.");
        return;
    }

    info!("Primary window already exists.");

    for mut window in &mut windows {
        window.visible = true;
    }
}

/// Determines the appropriate `PresentMode` based on the user's V-Sync configuration.
///
/// This function takes the V-Sync setting from the provided `ClientConfigs` structure
/// and maps it to an appropriate `PresentMode` enum variant. The mapping is case-insensitive
/// and allows for various string representations of V-Sync options, including aliases.
/// If the provided V-Sync option is unrecognized, a default value of `PresentMode::Fifo` is returned.
///
/// # Parameters
/// * `client_configs` - A reference to a `ClientConfigs` instance, which holds the configuration
///   data including the graphics settings, such as the V-Sync preference.
///
/// # Returns
/// * A `PresentMode` enum variant corresponding to the provided V-Sync configuration. The possible
///   mappings are as follows:
///   - `"vsync"` or `"true"` => `PresentMode::AutoVsync`
///   - `"novsync"` or `"false"` => `PresentMode::AutoNoVsync`
///   - `"default"` or `"fifo"` => `PresentMode::Fifo`
///   - `"fifo_relaxed"` => `PresentMode::FifoRelaxed`
///   - `"immediate"` => `PresentMode::Immediate`
///   - `"mailbox"` => `PresentMode::Mailbox`
///
/// # Notes
/// * The input string is converted to lowercase to ensure case-insensitive matching.
/// * This functionality is useful for selecting the appropriate swap chain presentation mode in graphics APIs.
fn vsync_mode(client_configs: &ClientConfigs) -> PresentMode {
    match client_configs
        .config_graphics
        .vsync
        .to_ascii_lowercase()
        .as_str()
    {
        "vsync" | "true" => PresentMode::AutoVsync,
        "novsync" | "false" => PresentMode::AutoNoVsync,
        "default" | "fifo" => PresentMode::Fifo,
        "fifo_relaxed" => PresentMode::FifoRelaxed,
        "immediate" => PresentMode::Immediate,
        "mailbox" => PresentMode::Mailbox,
        _ => PresentMode::Fifo,
    }
}
