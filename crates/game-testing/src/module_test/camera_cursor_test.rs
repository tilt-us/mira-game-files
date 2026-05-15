use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use game_shared::config::ClientConfigs;
use game_shared::models::camera::MenuCursorState;
use logic_module::camera_logic::camera_cursor::{
    apply_cursor_mode, init_cursor_lock_state, toggle_cursor_lock_with_open_menu,
};

fn cursor_state(app: &mut App) -> (bool, CursorGrabMode) {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&CursorOptions, With<PrimaryWindow>>();
    let cursor = query.iter(world).next().expect("expected cursor options");
    (cursor.visible, cursor.grab_mode)
}

#[test]
fn init_cursor_lock_state_hides_and_locks_cursor() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<MenuCursorState>();
    app.world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()));
    app.add_systems(Update, init_cursor_lock_state);

    app.update();

    let is_menu_open = app.world().resource::<MenuCursorState>().is_menu_open;
    let (visible, grab_mode) = cursor_state(&mut app);
    assert!(!is_menu_open);
    assert!(!visible);
    assert_eq!(grab_mode, CursorGrabMode::Locked);
}

#[test]
fn toggle_cursor_lock_with_open_menu_toggles_visibility_and_grab_mode() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(ClientConfigs::default());
    app.insert_resource(MenuCursorState::default());
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()));
    app.add_systems(Update, toggle_cursor_lock_with_open_menu);

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Escape);
    }
    app.update();
    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.release(KeyCode::Escape);
    }
    let (visible_open, grab_mode_open) = cursor_state(&mut app);
    assert!(app.world().resource::<MenuCursorState>().is_menu_open);
    assert!(visible_open);
    assert_eq!(grab_mode_open, CursorGrabMode::None);

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::Escape);
    }
    app.update();
    let (visible_closed, grab_mode_closed) = cursor_state(&mut app);
    assert!(!app.world().resource::<MenuCursorState>().is_menu_open);
    assert!(!visible_closed);
    assert_eq!(grab_mode_closed, CursorGrabMode::Locked);
}

#[test]
fn apply_cursor_mode_updates_all_primary_windows() {
    fn apply_closed(mut query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
        apply_cursor_mode(false, &mut query);
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()));
    app.world_mut()
        .spawn((PrimaryWindow, CursorOptions::default()));
    app.add_systems(Update, apply_closed);

    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<&CursorOptions, With<PrimaryWindow>>();
    for cursor in query.iter(world) {
        assert!(!cursor.visible);
        assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
    }
}
