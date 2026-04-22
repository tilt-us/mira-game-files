use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, Window, WindowMode, WindowResolution};

use crate::config::ClientConfigs;
use crate::states::ClientState;

pub struct ClientWindowPlugin;

impl Plugin for ClientWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::WindowVisible), create_or_show_primary_window);
    }
}

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
        return;
    }

    for mut window in &mut windows {
        window.visible = true;
    }
}

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
