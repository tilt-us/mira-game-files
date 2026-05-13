use crate::states::ClientState;
use bevy::prelude::*;
use logic_module::camera_logic::camera_init::init_camera;

pub struct CameraSystemComponent;

impl Plugin for CameraSystemComponent {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::WindowVisible), init_camera);
    }
}
