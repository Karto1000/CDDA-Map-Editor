use std::collections::HashMap;
use std::sync::Arc;

use bevy::math::{IVec2, Vec2};
use bevy::prelude::{Event, Resource};
use serde::{Deserialize, Serialize};

use crate::common::{Coordinates, MeabyWeighted, TileId};
use crate::common::GetRandom;
use crate::common::Weighted;
use crate::program::data::CDDAData;
use crate::map::io::{Parameter, ParameterId};
use crate::palettes::data::{MapObjectId, MeabyParam, PaletteId};
use crate::tiles::data::Tile;

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TileSelection {
    pub computed_parameters: ComputedParameters,
    pub parameters: HashMap<ParameterId, Parameter>,

    pub fill_ter: Option<TileId>,

    pub palettes: Vec<MapObjectId<MeabyParam>>,
    pub terrain: HashMap<char, MapObjectId<MeabyWeighted<MeabyParam>>>,
    pub furniture: HashMap<char, MapObjectId<MeabyWeighted<MeabyParam>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MapEntity {
    Single(Single),
    Multi(Multi),
    Nested(Nested),
}

#[derive(Debug, Default)]
pub struct TileIdGroup {
    pub terrain: Option<TileId>,
    pub furniture: Option<TileId>,
    pub toilet: Option<TileId>,
    pub item: Option<TileId>,
}

impl Default for MapEntity {
    fn default() -> Self {
        return Self::Single(Single::default());
    }
}


impl MapEntity {
    pub fn object(&self) -> &TileSelection {
        match self {
            MapEntity::Single(s) => &s.tile_selection,
            MapEntity::Multi(m) => &m.tile_selection,
            MapEntity::Nested(n) => &n.tile_selection
        }
    }

    pub fn object_mut(&mut self) -> &mut TileSelection {
        match self {
            MapEntity::Single(s) => &mut s.tile_selection,
            MapEntity::Multi(m) => &mut m.tile_selection,
            MapEntity::Nested(n) => &mut n.tile_selection
        }
    }

    pub fn tiles(&self) -> &HashMap<Coordinates, Tile> {
        match self {
            MapEntity::Single(s) => &s.tiles,
            MapEntity::Multi(m) => &m.tiles,
            MapEntity::Nested(n) => &n.tiles
        }
    }

    pub fn tiles_mut(&mut self) -> &mut HashMap<Coordinates, Tile> {
        match self {
            MapEntity::Single(s) => &mut s.tiles,
            MapEntity::Multi(m) => &mut m.tiles,
            MapEntity::Nested(n) => &mut n.tiles
        }
    }

    pub fn get_tiles_around(&self, coordinates: &Coordinates) -> Vec<(Option<&Tile>, Coordinates)> {
        let tiles = self.tiles();

        let top_coordinates = Coordinates { x: coordinates.x, y: coordinates.y - 1 };
        let right_coordinates = Coordinates { x: coordinates.x + 1, y: coordinates.y };
        let below_coordinates = Coordinates { x: coordinates.x, y: coordinates.y + 1 };
        let left_coordinates = Coordinates { x: coordinates.x - 1, y: coordinates.y };

        let tile_ontop = tiles.get(&top_coordinates);
        let tile_right = tiles.get(&right_coordinates);
        let tile_below = tiles.get(&below_coordinates);
        let tile_left = tiles.get(&left_coordinates);

        return vec![
            (tile_ontop, top_coordinates),
            (tile_right, right_coordinates),
            (tile_below, below_coordinates),
            (tile_left, left_coordinates),
        ];
    }

    pub fn size(&self) -> Vec2 {
        return match self {
            MapEntity::Single(s) => s.size.as_vec2(),
            MapEntity::Multi(_) => Vec2::new(24., 24.),
            MapEntity::Nested(n) => Vec2::new(n.om_terrain.len() as f32, n.row_size as f32)
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

        if let Some(id) = self.object().terrain.get(character) {
            match_id!(id, group.terrain, self.object().computed_parameters);
        }

        if let Some(id) = self.object().furniture.get(character) {
            match_id!(id, group.furniture, self.object().computed_parameters);
        }

        fn match_palette(map_entity: &MapEntity, cdda_data: &CDDAData, group: &mut TileIdGroup, character: &char, palette: &MapObjectId<MeabyParam>) {
            let palette_id = match palette {
                MapObjectId::Grouped(_) => { todo!() }
                MapObjectId::Nested(_) => { todo!() }
                MapObjectId::Param { param, fallback } => {
                    match map_entity.object().computed_parameters.get_value(param) {
                        None => fallback.as_ref().unwrap().clone(),
                        Some(v) => v.clone()
                    }
                }
                MapObjectId::Switch { .. } => { todo!() }
                MapObjectId::Single(mp) => {
                    match mp {
                        MeabyParam::TileId(id) => {
                            id.clone()
                        }
                        MeabyParam::Parameter(_) => { todo!() }
                    }
                }
            };

            let palette = cdda_data.palettes.get(&palette_id).unwrap();

            if let Some(id) = palette.furniture.get(character) {
                if group.furniture.is_none() {
                    match_id!(id, group.furniture, map_entity.object().computed_parameters);
                }
            }

            if let Some(id) = palette.terrain.get(character) {
                if group.terrain.is_none() {
                    match_id!(id, group.terrain, map_entity.object().computed_parameters);
                }
            }

            for parent_palette in palette.palettes.iter() {
                match_palette(map_entity, cdda_data, group, character, parent_palette);
            }
        }

        for palette_object_id in self.object().palettes.iter() {
            match_palette(self, cdda_data, &mut group, character, palette_object_id);
        }

        return group;
    }
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Single {
    pub om_terrain: String,
    pub tile_selection: TileSelection,
    pub tiles: HashMap<Coordinates, Tile>,
    pub size: IVec2
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Multi {
    pub om_terrain: Vec<String>,
    pub tile_selection: TileSelection,
    pub tiles: HashMap<Coordinates, Tile>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Nested {
    pub row_size: usize,
    pub om_terrain: Vec<String>,
    pub tile_selection: TileSelection,
    pub tiles: HashMap<Coordinates, Tile>,
}

#[derive(Event)]
pub struct UpdateSpriteEvent {
    pub tile: Tile,
    pub coordinates: Coordinates,
}

#[derive(Event, Debug)]
pub struct TilePlaceEvent {
    pub tile: Tile,
    pub coordinates: Coordinates,
    pub should_update_sprites: bool,
}

#[derive(Event, Debug)]
pub struct TileDeleteEvent {
    pub tile: Tile,
    pub coordinates: Coordinates,
}

#[derive(Event)]
pub struct SpawnMapEntity {
    pub map_entity: Arc<MapEntity>,
}

#[derive(Event)]
pub struct ClearTiles;
