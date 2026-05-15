use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::window::{PrimaryWindow, Window};

use game_client_package::ClientPackedPlugin;
use game_client_package::states::ClientState;
use game_shared::config::ClientConfigs;

fn primary_window_count(app: &mut App) -> usize {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Window, With<PrimaryWindow>>();
    query.iter(world).count()
}

fn create_app_with_client_packed_plugin() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.insert_resource(ClientConfigs::default());
    app.init_state::<ClientState>();
    app.add_plugins(ClientPackedPlugin);
    app
}

#[test]
fn client_packed_plugin_does_not_create_window_in_booting_state() {
    let mut app = create_app_with_client_packed_plugin();
    assert_eq!(primary_window_count(&mut app), 0);
}

#[test]
fn client_packed_plugin_keeps_required_resources_available() {
    let mut app = create_app_with_client_packed_plugin();

    assert!(app.world().contains_resource::<State<ClientState>>());
    assert!(app.world().contains_resource::<NextState<ClientState>>());
    assert!(app.world().contains_resource::<ClientConfigs>());
    assert_eq!(primary_window_count(&mut app), 0);
}
