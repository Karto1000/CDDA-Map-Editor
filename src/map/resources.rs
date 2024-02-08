use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::common::{Coordinates, MeabyWeighted, TileId};
use crate::palettes::{MapObjectId, Palette};
use crate::tiles::components::Tile;

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    pub om_terrain: String,
    pub weight: u32,
    pub tiles: HashMap<Coordinates, Tile>,
    pub size: Vec2,

    pub palettes: Vec<Palette>,
}

impl Default for MapEntity {
    fn default() -> Self {
        return Self {
            om_terrain: "unnamed".into(),
            weight: 0,
            tiles: HashMap::new(),
            size: Vec2::new(24., 24.),
            palettes: Vec::new(),
        };
    }
}

impl MapEntity {
    pub fn new(name: String, size: Vec2) -> Self {
        return Self {
            om_terrain: name,
            weight: 100,
            tiles: HashMap::new(),
            size,
            palettes: Vec::new(),
        };
    }

    pub fn get_tile_id_from_character(&self, character: &char) -> TileId {
        for palette in self.palettes.iter() {
            if let Some(id) = palette.terrain.get(character) {
                match id {
                    MapObjectId::Single(v) => {
                        match v {
                            MeabyWeighted::NotWeighted(v) => {
                                return v.clone();
                            }
                            MeabyWeighted::Weighted(_) => { panic!("Not Implemented") }
                        }
                    }
                    MapObjectId::Grouped(_) => { panic!("Not Implemented") }
                    MapObjectId::Nested(_) => { panic!("Not Implemented") }
                }
            }
        }

        return TileId { 0: "TODO_IMPLEMENT_DEFAULT".into() };
    }

    pub fn get_tiles_around(&self, coordinates: &Coordinates) -> Vec<(Option<&Tile>, Coordinates)> {
        let top_coordinates = Coordinates { x: coordinates.x, y: coordinates.y - 1 };
        let right_coordinates = Coordinates { x: coordinates.x + 1, y: coordinates.y };
        let below_coordinates = Coordinates { x: coordinates.x, y: coordinates.y + 1 };
        let left_coordinates = Coordinates { x: coordinates.x - 1, y: coordinates.y };

        let tile_ontop = self.tiles.get(&top_coordinates);
        let tile_right = self.tiles.get(&right_coordinates);
        let tile_below = self.tiles.get(&below_coordinates);
        let tile_left = self.tiles.get(&left_coordinates);

        return vec![
            (tile_ontop, top_coordinates),
            (tile_right, right_coordinates),
            (tile_below, below_coordinates),
            (tile_left, left_coordinates),
        ];
    }
}