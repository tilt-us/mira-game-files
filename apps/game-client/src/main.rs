use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings, WgpuSettingsPriority};
use game_client_package::ClientPackedPlugin;
use game_client_package::config::ClientConfigs;
use game_client_package::config::debug_config::AppBuildInfo;
use game_client_package::states::{ClientState, LoadingState};

fn main() {
    let client_config = ClientConfigs::new();
    let mut app = App::new();
    boot_bevy_app(&mut app, &client_config);
}

fn boot_bevy_app(app: &mut App, config: &ClientConfigs) {
    let app_build_info = AppBuildInfo {
        name: "Mira: Fallen Gates".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        bevy_version: "0.18.1".to_string(),
    };

    app.insert_resource(config.clone());
    app.insert_resource(app_build_info);

    app.add_plugins(DefaultPlugins
        .set(
            WindowPlugin {
                primary_window: None,
                ..default()
            }
        )
        .set(
        RenderPlugin {
            render_creation: RenderCreation::Automatic(select_gpu_backend(config.clone())),
            ..default()
        }
        )
        .set(ImagePlugin::default_nearest()
        ));

    app.add_plugins(ClientPackedPlugin);

    app.init_state::<ClientState>();

    app.add_systems(Startup, | mut next_state: ResMut<NextState<ClientState>>| {
        next_state.set(ClientState::Loading(LoadingState::UiPreLoad));
    });

    app.run();
}

fn select_gpu_backend(client_configs: ClientConfigs) -> WgpuSettings {
    let backend = match client_configs.config_graphics.graphic_backend.to_ascii_lowercase().as_str() {
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
