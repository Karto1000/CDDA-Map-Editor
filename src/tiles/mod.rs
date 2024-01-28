use bevy::app::{App, Plugin, Update};
use bevy::math::Vec2;
use bevy::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};

use crate::tiles::systems::{tile_delete_system, tile_place_system, tile_resize_system, window_tile_resize_system};
use crate::tiles::resources::PlaceInfo;

pub(crate) mod systems;
pub(crate) mod resources;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        let place_info: PlaceInfo = PlaceInfo {
            last_place_position: None
        };

        app.insert_resource(place_info);
        app.add_systems(Update, (window_tile_resize_system, tile_resize_system, tile_place_system, tile_delete_system));
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct Tile {
    pub tile_type: TileType,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum TileType {
    Test
}

