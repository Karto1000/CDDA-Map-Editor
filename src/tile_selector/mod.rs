pub(crate) mod build;

use bevy::app::{App, Plugin, Startup, Update};
use crate::tile_selector::build::{spawn_tile_selector};

pub struct TileSelectorPlugin;

impl Plugin for TileSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tile_selector);
    }
}