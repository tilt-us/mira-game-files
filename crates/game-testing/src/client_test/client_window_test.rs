use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, Window, WindowMode};

use game_client_package::client_window::ClientWindowPlugin;
use game_client_package::states::ClientState;
use game_shared::config::ClientConfigs;

fn window_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
    query.iter(world).count()
}

fn primary_window(app: &mut App) -> Window {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
    query
        .iter(world)
        .next()
        .expect("expected a primary window")
        .clone()
}

fn run_window_visible_once(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<ClientState>>()
        .set(ClientState::WindowVisible);
    app.update();
}

#[test]
fn creates_primary_window_for_window_visible_state() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.insert_resource(ClientConfigs::default());
    app.init_state::<ClientState>();
    app.add_plugins(ClientWindowPlugin);

    run_window_visible_once(&mut app);

    let window = primary_window(&mut app);
    assert_eq!(window_count(&mut app), 1);
    assert_eq!(window.title, "Mira: Fallen Gates");
    assert!(matches!(window.mode, WindowMode::Windowed));
    assert_eq!(window.resolution.width(), 1270.0);
    assert_eq!(window.resolution.height(), 720.0);
    assert!(window.visible);
    assert_eq!(window.present_mode, PresentMode::Fifo);
}

#[test]
fn shows_existing_primary_window_instead_of_spawning_new_one() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.insert_resource(ClientConfigs::default());
    app.init_state::<ClientState>();
    app.add_plugins(ClientWindowPlugin);
    app.world_mut().spawn((
        Window {
            visible: false,
            ..default()
        },
        PrimaryWindow,
    ));

    run_window_visible_once(&mut app);

    let window = primary_window(&mut app);
    assert_eq!(window_count(&mut app), 1);
    assert!(window.visible);
}

#[test]
fn creates_fullscreen_window_and_maps_vsync_values() {
    let mut config = ClientConfigs::default();
    config.config_graphics.fullscreen = true;
    config.config_graphics.vsync = String::from("mailbox");

    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.insert_resource(config);
    app.init_state::<ClientState>();
    app.add_plugins(ClientWindowPlugin);

    run_window_visible_once(&mut app);

    let window = primary_window(&mut app);
    assert!(matches!(
        window.mode,
        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
    ));
    assert_eq!(window.present_mode, PresentMode::Mailbox);
}

#[test]
fn falls_back_to_fifo_for_unknown_vsync() {
    let mut config = ClientConfigs::default();
    config.config_graphics.vsync = String::from("not-a-mode");

    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.insert_resource(config);
    app.init_state::<ClientState>();
    app.add_plugins(ClientWindowPlugin);

    run_window_visible_once(&mut app);

    let window = primary_window(&mut app);
    assert_eq!(window.present_mode, PresentMode::Fifo);
}
