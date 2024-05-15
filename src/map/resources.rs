use std::collections::HashMap;
use bevy::prelude::{Resource, Vec2};
use serde::{Deserialize, Serialize};
use crate::common::{Coordinates, MeabyMulti, MeabyNumberRange, MeabyWeighted, TileId};
use crate::map::loader::ParameterId;
use crate::palettes::{Item, MapObjectId, PaletteId};
use crate::tiles::components::Tile;
use crate::{MeabyParam};
use crate::Weighted;
use crate::common::GetRandom;
use crate::editor_data::CDDAData;

#[derive(Default, Serialize, Deserialize, Debug, Resource, Clone)]
pub struct ComputedParameters {
    pub this: HashMap<ParameterId, String>,
    pub palettes: HashMap<PaletteId, ComputedParameters>,
}

impl ComputedParameters {
    pub fn get_value(&self, parameter_id: &String) -> Option<&String> {
        match self.this.get(parameter_id) {
            None => {
                for (_, parameters) in self.palettes.iter() {
                    match parameters.get_value(parameter_id) {
                        None => {}
                        Some(v) => return Some(v)
                    }
                }
            }
            Some(v) => return Some(v)
        };

        return None;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MapEntityType {
    NestedMapgen {
        nested_mapgen_id: String,
    },
    Default {
        om_terrain: String,
        weight: u32,
    },
    Multi {
        om_terrain: Vec<String>,
        weight: u32,
    },
    Nested {
        om_terrain: Vec<Vec<String>>,
        weight: u32,
    },
}

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

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    #[serde(flatten)]
    pub map_type: MapEntityType,
    pub tiles: HashMap<Coordinates, Tile>,
    pub size: Vec2,

    #[serde(skip)]
    pub fill: Option<TileId>,

    #[serde(skip)]
    pub computed_parameters: ComputedParameters,

    #[serde(default)]
    pub terrain: HashMap<char, MapObjectId<MeabyParam>>,

    #[serde(default)]
    pub furniture: HashMap<char, MapObjectId<MeabyParam>>,

    #[serde(default)]
    pub items: HashMap<char, MeabyMulti<Item>>,

    #[serde(default)]
    pub place_nested: Vec<PlaceNested>,

    #[serde(default)]
    pub palettes: Vec<MapObjectId<MeabyParam>>,
}

impl Default for MapEntity {
    fn default() -> Self {
        return Self {
            map_type: MapEntityType::Default {
                om_terrain: "unnamed_01".to_string(),
                weight: 1000,
            },
            fill: None,
            tiles: Default::default(),
            size: Vec2 { x: 24., y: 24. },
            computed_parameters: ComputedParameters { this: Default::default(), palettes: Default::default() },
            terrain: Default::default(),
            furniture: Default::default(),
            items: Default::default(),
            place_nested: vec![],
            palettes: vec![],
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
            fill: None,
            tiles: HashMap::new(),
            size,
            computed_parameters: ComputedParameters { this: Default::default(), palettes: Default::default() },
            terrain: Default::default(),
            furniture: Default::default(),
            palettes: Vec::new(),
            place_nested: Vec::new(),
            items: Default::default(),
        };
    }

    pub fn get_ids(&self, cdda_data: &CDDAData, character: &char) -> TileIdGroup {
        let mut group = TileIdGroup::default();

        macro_rules! match_id {
            ($id: ident, $path: expr, $computed_parameters: expr) => {
                match $id {
                    MapObjectId::Single(v) => {
                        match v {
                            MeabyWeighted::NotWeighted(v) => {
                                let id = match v {
                                    MeabyParam::TileId(id) => id,
                                    MeabyParam::Parameter(parameter) => todo!()
                                };
                                $path = Some(id.clone());
                            }
                            MeabyWeighted::Weighted(_) => todo!()
                        }
                    }
                    MapObjectId::Grouped(g) => {
                        let final_group: Vec<Weighted<MeabyParam>> = g.iter().map(|mw| {
                            match mw {
                                MeabyWeighted::NotWeighted(v) => Weighted::new(v.clone(), 1),
                                MeabyWeighted::Weighted(w) => w.clone()
                            }
                        }).collect();

                        let random_sprite = final_group.get_random_weighted().unwrap();
                        let id = match &random_sprite {
                            MeabyParam::TileId(id) => id,
                            MeabyParam::Parameter(parameter) => todo!()
                        };

                        $path = Some(id.clone());
                    }
                    MapObjectId::Nested(_) => todo!(),
                    MapObjectId::Param { param, fallback } => {
                        $path = Some($computed_parameters.get_value(param).expect(format!("Parameter {} to exist", param).as_str()).clone());
                    }
                    MapObjectId::Switch {switch, cases} => todo!(),
                }
            }
        }

        if let Some(id) = self.terrain.get(character) {
            match_id!(id, group.terrain, self.computed_parameters);
        }

        if let Some(id) = self.furniture.get(character) {
            match_id!(id, group.furniture, self.computed_parameters);
        }

        fn match_palette(map_entity: &MapEntity, cdda_data: &CDDAData, group: &mut TileIdGroup, character: &char, palette: &MapObjectId<MeabyParam>) {
            let palette_id = match palette {
                MapObjectId::Grouped(_) => { todo!() }
                MapObjectId::Nested(_) => { todo!() }
                MapObjectId::Param { param, fallback } => {
                    match map_entity.computed_parameters.get_value(param) {
                        None => fallback.as_ref().unwrap().clone(),
                        Some(v) => v.clone()
                    }
                }
                MapObjectId::Switch { .. } => { todo!() }
                MapObjectId::Single(mw) => {
                    match mw {
                        MeabyWeighted::NotWeighted(i) => {
                            match i {
                                MeabyParam::TileId(id) => {
                                    id.clone()
                                }
                                MeabyParam::Parameter(_) => { todo!() }
                            }
                        }
                        MeabyWeighted::Weighted(_) => { todo!() }
                    }
                }
            };

            let palette = cdda_data.palettes.get(&palette_id).unwrap();

            if let Some(id) = palette.furniture.get(character) {
                if group.furniture.is_none() {
                    match_id!(id, group.furniture, map_entity.computed_parameters);
                }
            }

            if let Some(id) = palette.terrain.get(character) {
                if group.terrain.is_none() {
                    match_id!(id, group.terrain, map_entity.computed_parameters);
                }
            }

            for parent_palette in palette.palettes.iter() {
                match_palette(map_entity, cdda_data, group, character, parent_palette);
            }
        }

        for palette_object_id in self.palettes.iter() {
            match_palette(self, cdda_data, &mut group, character, palette_object_id);
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