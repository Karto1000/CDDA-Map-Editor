pub(crate) mod systems;

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component)]
pub struct Tile {
    pub tile_type: TileType,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TileType {
    Test
}

