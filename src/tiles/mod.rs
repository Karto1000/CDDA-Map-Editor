use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};

use crate::tiles::resources::PlaceInfo;
use crate::tiles::systems::{tile_delete_system, tile_place_system, tile_resize_system, window_tile_resize_system};

pub(crate) mod systems;
pub(crate) mod resources;
pub(crate) mod components;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        let place_info: PlaceInfo = PlaceInfo {
            last_place_position: None,
        };

        app.insert_resource(place_info);
        app.add_systems(Update, (
            window_tile_resize_system,
            tile_resize_system,
            tile_place_system,
            tile_delete_system
        ));
    }
}

