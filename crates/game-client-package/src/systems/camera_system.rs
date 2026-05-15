use crate::states::ClientState;
use bevy::prelude::*;
use game_shared::models::camera::MenuCursorState;
use logic_module::camera_logic::camera_cursor::{
    init_cursor_lock_state, toggle_cursor_lock_with_open_menu,
};
use logic_module::camera_logic::camera_follow::{
    follow_player_orbit_camera, init_orbit_follow_camera,
};
use logic_module::camera_logic::camera_init::init_camera;

pub struct CameraSystemComponent;

impl Plugin for CameraSystemComponent {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuCursorState>();
        app.add_systems(
            OnEnter(ClientState::WindowVisible),
            (
                init_camera,
                init_orbit_follow_camera.after(init_camera),
                init_cursor_lock_state,
            ),
        );
        app.add_systems(
            Update,
            (
                follow_player_orbit_camera,
                toggle_cursor_lock_with_open_menu,
            )
                .run_if(in_state(ClientState::WindowVisible)),
        );
    }
}
