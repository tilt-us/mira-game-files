use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use game_shared::config::ClientConfigs;
use game_shared::models::camera::MenuCursorState;
use game_shared::utils::input_utils::detect_key_action;

/// Applies cursor visibility and lock mode based on the menu state.
///
/// # Parameters
/// - `is_menu_open`: Whether the in-game menu is currently open.
/// - `cursor_query`: Query for cursor options on the primary window.
///
/// # Behavior
/// - When the menu is open, the cursor is shown and unlocked.
/// - When the menu is closed, the cursor is hidden and locked to the window.
pub fn apply_cursor_mode(
    is_menu_open: bool,
    cursor_query: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    for mut cursor_options in cursor_query.iter_mut() {
        if is_menu_open {
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        } else {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
        }
    }
}

/// Initializes the cursor lock state for the menu system.
///
/// This function is responsible for setting the initial state of the menu cursor
/// and applying the default cursor mode (unlocked).
///
/// # Arguments
///
/// * `state` - A mutable resource of type [`MenuCursorState`], which tracks the
///   state of the menu, particularly whether the menu is open or closed.
/// * `cursor_query` - A mutable query for fetching components of type [`CursorOptions`]
///   associated with entities that have the [`PrimaryWindow`] component. This is used
///   to manage and modify the cursor's behavior in the primary game window.
///
/// # Behavior
///
/// * Sets the `is_menu_open` property of `MenuCursorState` to `false`, indicating that
///   the menu is closed.
/// * Calls the `apply_cursor_mode` function to disable the cursor lock, making the cursor
///   freely movable within the primary window.
pub fn init_cursor_lock_state(
    mut state: ResMut<MenuCursorState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    state.is_menu_open = false;
    apply_cursor_mode(false, &mut cursor_query);
}

/// Toggles the cursor lock state and handles the state of the "open menu" functionality.
///
/// This function listens for a specific key action based on the user-defined configuration to open or close the menu.
/// When the menu state toggles, it updates the cursor mode accordingly.
///
/// # Parameters
///
/// - `keyboard`: A resource that tracks the current state of keyboard inputs (`Res<ButtonInput<KeyCode>>`).
/// - `client_configs`: A resource containing the client configuration settings (`Res<ClientConfigs>`).
/// - `state`: A mutable resource (`ResMut<MenuCursorState>`) that holds the current state of the menu and cursor lock.
/// - `cursor_query`: A query to fetch and modify entities with the `CursorOptions` component, specifically for the primary window (`Query<&mut CursorOptions, With<PrimaryWindow>>`).
///
/// # Behavior
///
/// - Detects if the key action associated with "open menu" (defined in the client configuration) has been triggered.
/// - Toggles the `is_menu_open` flag in the `MenuCursorState`.
/// - Updates the cursor mode through `apply_cursor_mode` based on the new menu state.
///
/// # Usage
///
/// This function is expected to be called on each update cycle within a game or application loop.
/// It ensures that the cursor's lock state and the menu state remain synchronized based on user interaction.
///
/// # See Also
///
/// - [`detect_key_action`](#): Handles key event detection for a specified action.
/// - [`apply_cursor_mode`](#): Applies the appropriate cursor mode based on the menu state.
pub fn toggle_cursor_lock_with_open_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    client_configs: Res<ClientConfigs>,
    mut state: ResMut<MenuCursorState>,
    mut cursor_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if detect_key_action(
        client_configs.config_input.open_menu().to_string(),
        &keyboard,
    ) {
        state.is_menu_open = !state.is_menu_open;
    }

    apply_cursor_mode(state.is_menu_open, &mut cursor_query);
}
