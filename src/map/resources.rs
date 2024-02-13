use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::Resource;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::common::{Coordinates, MeabyNumberRange, MeabyWeighted, TileId, Weighted};
use crate::palettes::{MapObjectId, Palette};
use crate::tiles::components::Tile;

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
#[serde(untagged)]
pub enum PlaceNested {
    Includes {
        chunks: Vec<MeabyWeighted<String>>,
        x: MeabyNumberRange<i32>,
        y: MeabyNumberRange<i32>,
    },
    Exclude {
        else_chunks: Vec<MeabyWeighted<String>>,
        x: MeabyNumberRange<i32>,
        y: MeabyNumberRange<i32>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MapEntityType {
    Nested {
        nested_mapgen_id: String,
    },
    Default {
        om_terrain: String,
        weight: u32,
    },
}

impl MapEntityType {
    pub fn get_name(&self) -> &String {
        return match self {
            MapEntityType::Nested { nested_mapgen_id, .. } => nested_mapgen_id,
            MapEntityType::Default { om_terrain, .. } => om_terrain
        };
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            MapEntityType::Nested { ref mut nested_mapgen_id, .. } => { *nested_mapgen_id = name }
            MapEntityType::Default { ref mut om_terrain, .. } => { *om_terrain = name }
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileEntity {
    terrain: Option<Tile>,
    furniture: Option<Tile>
}

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    #[serde(flatten)]
    pub map_type: MapEntityType,
    pub tiles: HashMap<Coordinates, Tile>,
    pub size: Vec2,

    pub place_nested: Option<Vec<PlaceNested>>,
    pub palettes: Vec<Palette>,
}

impl Default for MapEntity {
    fn default() -> Self {
        return Self {
            map_type: MapEntityType::Default {
                om_terrain: "unnamed".into(),
                weight: 0,
            },
            tiles: HashMap::new(),
            size: Vec2::new(24., 24.),
            palettes: Vec::new(),
            place_nested: None,
        };
    }
}

impl MapEntity {
    pub fn new(name: String, size: Vec2) -> Self {
        return Self {
            map_type: MapEntityType::Default {
                om_terrain: name,
                weight: 100,
            },
            tiles: HashMap::new(),
            size,
            palettes: Vec::new(),
            place_nested: None,
        };
    }

    pub fn get_terrain_id_from_character(&self, character: &char) -> TileId {
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
                    MapObjectId::Param { .. } => { panic!("Not Implemented") }
                }
            }
        }

        return TileId { 0: "TODO_IMPLEMENT_DEFAULT".into() };
    }

    pub fn get_furniture_id_from_character(&self, character: &char) -> TileId {
        for palette in self.palettes.iter() {
            if let Some(id) = palette.furniture.get(character) {
                match id {
                    MapObjectId::Single(v) => {
                        match v {
                            MeabyWeighted::NotWeighted(v) => {
                                return v.clone();
                            }
                            MeabyWeighted::Weighted(_) => { panic!("Not Implemented") }
                        }
                    }
                    MapObjectId::Grouped(g) => {
                        let final_group: Vec<Weighted<TileId>> = g.iter().map(|mw| {
                            match mw {
                                MeabyWeighted::NotWeighted(v) => Weighted::new(v.clone(), 1),
                                MeabyWeighted::Weighted(w) => w.clone()
                            }
                        }).collect();

                        // TODO Take weights into account
                        let mut rng = rand::thread_rng();
                        let random_index: usize = rng.gen_range(0..final_group.len());
                        let random_sprite = final_group.get(random_index).unwrap();
                        return random_sprite.value.clone();
                    }
                    MapObjectId::Nested(_) => { panic!("Not Implemented") }
                    MapObjectId::Param { .. } => { panic!("Not Implemented") }
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