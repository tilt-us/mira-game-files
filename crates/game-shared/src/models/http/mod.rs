pub mod account;
pub mod character;
pub mod team;

use crate::models::http::account::AccountResource;
use bevy::prelude::*;

pub struct HttpApp;

impl Plugin for HttpApp {
    fn build(&self, app: &mut App) {
        app.init_resource::<AccountResource>();
    }
}
