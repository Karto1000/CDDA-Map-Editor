use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::{EventWriter, Resource};
use serde::{Deserialize, Serialize};

use crate::common::{MeabyWeighted, TileId};
use crate::map::{Coordinates, TilePlaceEvent};
use crate::palettes::{MapObjectId, Palette};
use crate::tiles::Tile;

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

    pub fn set_tile_at(
        &mut self,
        character: char,
        cords: (i32, i32),
        e_set_tile: &mut EventWriter<TilePlaceEvent>,
    ) {
        let coordinates = Coordinates { x: cords.0, y: cords.1 };
        if self.tiles.get(&coordinates).is_some() { return; }

        let tile = Tile { character, x: cords.0, y: cords.1, entity: None };

        e_set_tile.send(TilePlaceEvent { tile, update_sprites_around: true });

        self.tiles.insert(
            coordinates,
            tile,
        );
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

    pub fn get_tiles_around(&self, coordinates: &Coordinates) -> Vec<Option<&Tile>> {
        let tile_ontop = self.tiles.get(&Coordinates { x: coordinates.x, y: coordinates.y - 1 });
        let tile_right = self.tiles.get(&Coordinates { x: coordinates.x + 1, y: coordinates.y });
        let tile_below = self.tiles.get(&Coordinates { x: coordinates.x, y: coordinates.y + 1 });
        let tile_left = self.tiles.get(&Coordinates { x: coordinates.x - 1, y: coordinates.y });
        return vec![tile_ontop, tile_right, tile_below, tile_left];
    }
}