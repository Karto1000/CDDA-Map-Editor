use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::{default, Resource};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::common::{Coordinates, GetRandom, MeabyMulti, MeabyNumberRange, MeabyWeighted, TileId, Weighted};
use crate::palettes::{Identifier, Item, MapObjectId, Palette, Parameter, ParentPalette};
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

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    #[serde(flatten)]
    pub map_type: MapEntityType,
    pub tiles: HashMap<Coordinates, Tile>,
    pub size: Vec2,

    #[serde(default)]
    pub parameters: HashMap<String, Parameter>,

    #[serde(default)]
    pub terrain: HashMap<char, MapObjectId>,

    #[serde(default)]
    pub furniture: HashMap<char, MapObjectId>,

    #[serde(default)]
    pub items: HashMap<char, MeabyMulti<Item>>,

    #[serde(default)]
    pub place_nested: Vec<PlaceNested>,

    #[serde(default)]
    pub palettes: Vec<Palette>,
}

impl MapEntity {
    pub fn add_palette(&mut self, all_palettes: &HashMap<String, Palette>, palette: &Palette) {
        let mut computed_palette = palette.clone();

        // Compute parameters
        for (_, parameter) in computed_palette.parameters.iter_mut() {
            parameter.calculated_value = Some(parameter.default.get_value());
        }

        computed_palette.compute_parent_palettes(all_palettes);

        self.palettes.push(computed_palette);
    }
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
            parameters: Default::default(),
            terrain: Default::default(),
            furniture: Default::default(),
            palettes: Vec::new(),
            place_nested: Vec::new(),
            items: Default::default(),
        };
    }
}

#[derive(Debug)]
pub struct TileIdGroup {
    pub terrain: Option<TileId>,
    pub furniture: Option<TileId>,
    pub toilet: Option<TileId>,
    pub item: Option<TileId>,
}

impl Default for TileIdGroup {
    fn default() -> Self {
        return Self {
            terrain: None,
            furniture: None,
            toilet: None,
            item: None,
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
            parameters: Default::default(),
            terrain: Default::default(),
            furniture: Default::default(),
            palettes: Vec::new(),
            place_nested: Vec::new(),
            items: Default::default(),
        };
    }

    pub fn get_ids(&self, character: &char) -> TileIdGroup {
        let mut group = TileIdGroup::default();

        macro_rules! match_id {
            ($id: ident, $path: expr, $obj_with_parameter: ident) => {
                match $id {
                    MapObjectId::Single(v) => {
                        match v {
                            MeabyWeighted::NotWeighted(v) => {
                                let id = match v {
                                    Identifier::TileId(id) => id,
                                    Identifier::Parameter(parameter) => {
                                        println!("{:?} {:?} {:?} {}", parameter.param, parameter.fallback, $id, character);
                                        panic!("Not Implemented 1")
                                    }
                                };
                                $path = Some(id.clone());
                            }
                            MeabyWeighted::Weighted(_) => { panic!("Not Implemented") }
                        }
                    }
                    MapObjectId::Grouped(g) => {
                        let final_group: Vec<Weighted<Identifier>> = g.iter().map(|mw| {
                            match mw {
                                MeabyWeighted::NotWeighted(v) => Weighted::new(v.clone(), 1),
                                MeabyWeighted::Weighted(w) => w.clone()
                            }
                        }).collect();

                        let random_sprite = final_group.get_random_weighted().unwrap();
                        let id = match &random_sprite {
                            Identifier::TileId(id) => id,
                            Identifier::Parameter(parameter) => {
                                println!("{}", parameter.param);
                                panic!("Not Implemented Pid")
                            }
                        };

                        $path = Some(id.clone());
                    }
                    MapObjectId::Nested(_) => { panic!("Not Implemented") }
                    MapObjectId::Param { param, fallback } => {
                        let mut parameter = $obj_with_parameter.parameters.get(param).expect(format!("Parameter {} to exist", param).as_str());

                        match &parameter.calculated_value {
                            Some(v) => {
                                $path = Some(v.clone())
                            }
                            None => {
                                panic!("Value was not calculated for parameter {}", param);
                            }
                        }
                    }
                    MapObjectId::Switch {switch, cases} => {
                        println!("{:?} {:?}", switch, cases);
                        panic!("Not Implemented")
                    }
                }
            }
        }

        if let Some(id) = self.terrain.get(character) {
            match_id!(id, group.terrain, self);
        }

        if let Some(id) = self.furniture.get(character) {
            match_id!(id, group.furniture, self);
        }

        for palette in self.palettes.iter() {
            if let Some(id) = palette.furniture.get(character) {
                match_id!(id, group.furniture, palette);
            }

            if let Some(id) = palette.terrain.get(character) {
                match_id!(id, group.terrain, palette);
            }

            for parent_palette in palette.palettes.iter() {
                match parent_palette {
                    ParentPalette::NotComputed(_) => { panic!() }
                    ParentPalette::Computed(p) => {
                        if let Some(id) = p.furniture.get(character) {
                            match_id!(id, group.furniture, p);
                        }

                        if let Some(id) = p.terrain.get(character) {
                            match_id!(id, group.terrain, p);
                        }
                    }
                }
            }
        }

        return group;
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