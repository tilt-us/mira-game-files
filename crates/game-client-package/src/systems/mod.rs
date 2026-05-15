mod camera_system;
mod player_system;
mod world_system;

use crate::systems::camera_system::CameraSystemComponent;
use crate::systems::player_system::PlayerSystemComponent;
use crate::systems::world_system::WorldSystemComponent;
use bevy::prelude::*;

pub struct ClientSystemPlugin;

impl Plugin for ClientSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraSystemComponent,
            PlayerSystemComponent,
            WorldSystemComponent,
        ));
    }
}
