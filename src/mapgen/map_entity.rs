use std::collections::HashMap;

use bevy::prelude::{Component, Resource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component)]
pub struct Tile {
    pub tile_type: TileType,
    pub x: i32,
    pub y: i32
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TileType {
    Test
}

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub tiles: HashMap<(i32, i32), Tile>,
}

impl MapEntity {
    pub fn get_size(&self) -> Option<(i32, i32)> {
        let mut keys: Vec<(i32, i32)> = self.tiles.clone().into_keys().collect();
        keys.sort();
        return keys.last().cloned();
    }

    pub fn new(name: String) -> Self {
        return Self {
            name,
            weight: 100,
            tiles: HashMap::new(),
        };
    }

    pub fn set_tile_at(&mut self, cords: (i32, i32), tile_type: TileType) {
        self.tiles.insert(
            cords,
            Tile { tile_type, x: cords.0, y: cords.1 },
        );
    }
}