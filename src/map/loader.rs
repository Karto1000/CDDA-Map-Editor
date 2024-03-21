use std::collections::HashMap;
use std::path::PathBuf;

use bevy::math::Vec2;
use bevy::prelude::Resource;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ALL_PALETTES;
use crate::common::{Coordinates, MeabyMulti, MeabyNumberRange, MeabyWeighted, TileId};
use crate::common::io::{Load, LoadError};
use crate::map::resources::MapEntity;
use crate::palettes::{Identifier, Item, MapGenValue, MapObjectId, PaletteId, ParameterType};
use crate::tiles::components::Tile;
use crate::map::resources::ComputedParameters;
use crate::map::resources::MapEntityType;

/**
 * Copyright (c) YPSOMED AG, Burgdorf / Switzerland
 * YDS INNOVATION - Digital Innovation
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * This project is for demonstration purposes only and can not be used
 * for medical purposes or in a production environment.
 *
 * email: diginno@ypsomed.com
 * author: Tim Leuenberger (Tim.leuenberger@ypsomed.com)
 */

pub type ParameterId = String;

pub struct MapEntityLoader {
    pub path: PathBuf,
    pub id: String,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub default: MapGenValue,
}

fn compute_palettes(parameters: &HashMap<String, String>, palettes: &Vec<MapObjectId>) -> HashMap<PaletteId, ComputedParameters> {
    let mut computed_palettes = HashMap::new();

    for palette in palettes.iter() {
        let palette_id: PaletteId = match palette {
            MapObjectId::Grouped(_) => { todo!() }
            MapObjectId::Nested(_) => { todo!() }
            MapObjectId::Param { param, fallback } => {
                match parameters.get(param) {
                    None => fallback.as_ref().unwrap().clone(),
                    Some(v) => v.clone()
                }
            }
            MapObjectId::Switch { .. } => { todo!() }
            MapObjectId::Single(o) => {
                match o {
                    MeabyWeighted::NotWeighted(i) => {
                        match i {
                            Identifier::TileId(i) => {
                                i.0.clone()
                            }
                            Identifier::Parameter(_) => { todo!() }
                        }
                    }
                    MeabyWeighted::Weighted(_) => { todo!() }
                }
            }
        };

        let associated_palette = ALL_PALETTES.get(&palette_id).unwrap();

        let mut this = HashMap::new();

        for (name, parameter) in associated_palette.parameters.iter() {
            this.insert(name.clone(), parameter.default.get_value().0);
        }

        let computed_palette_parameters = ComputedParameters {
            this: this.clone(),
            palettes: compute_palettes(&this, &associated_palette.palettes),
        };

        computed_palettes.insert(palette_id, computed_palette_parameters);

        info!("Computed Parameters for {:?}", palette)
    }

    return computed_palettes;
}

impl Load<MapEntity> for MapEntityLoader {
    fn load(&self) -> Result<MapEntity, LoadError> {
        let mut om_terrain = "".to_string();
        
        let objects = serde_json::from_str::<Vec<HashMap<String, Value>>>(std::fs::read_to_string(&self.path).unwrap().as_str()).unwrap();
        let filtered_objects = objects
            .into_iter()
            .filter(|o| {
                return match o.get("om_terrain") {
                    None => false,
                    Some(s) => match s {
                        Value::String(s) => {
                            om_terrain = s.clone();
                            s.clone() == self.id
                        },
                        Value::Array(a) => {
                            // TODO actually implement this correctly
                            let first = a.first().unwrap();
                            let string = first.as_str().unwrap().to_string();
                            om_terrain = string.clone();
                            string == self.id
                        },
                        _ => panic!()
                    }
                };
            })
            .collect::<Vec<HashMap<String, Value>>>();

        let mapgen_entity = filtered_objects.first().unwrap();
        let object = mapgen_entity.get("object").unwrap();


        let parameters = match object.get("parameters") {
            None => HashMap::new(),
            Some(v) => serde_json::from_value::<HashMap<ParameterId, Parameter>>(v.clone()).unwrap()
        };

        let rows: Vec<String> = serde_json::from_value(object.get("rows").unwrap().clone()).unwrap();

        let mut tiles = HashMap::new();

        let size = Vec2::new(
            rows.get(0).unwrap().as_str().len() as f32,
            rows.len() as f32,
        );

        for (row, tile) in rows.iter().enumerate() {
            // to_string returns quotes so we use as_str
            for (column, char) in tile.as_str().chars().enumerate() {
                tiles.insert(
                    Coordinates::new(column as i32, row as i32),
                    Tile::from(char),
                );
            }
        }

        let palettes: Vec<MapObjectId> = serde_json::from_value(object.get("palettes").unwrap_or(&Value::Array(Vec::new())).clone()).unwrap();

        let mut this = HashMap::new();

        for (parameter_id, parameter) in parameters.iter() {
            this.insert(
                parameter_id.clone(),
                parameter.default.get_value().0,
            );
        }

        let computed_parameters = ComputedParameters {
            this: this.clone(),
            palettes: compute_palettes(&this, &palettes),
        };

        let terrain = match object.get("terrain") {
            None => HashMap::new(),
            Some(t) => serde_json::from_value(t.clone()).unwrap()
        };

        let furniture = match object.get("furniture") {
            None => HashMap::new(),
            Some(f) => serde_json::from_value(f.clone()).unwrap()
        };

        let fill: Option<TileId> = match object.get("fill_ter") {
            None => None,
            Some(v) => Some(TileId(v.as_str().unwrap().to_string()))
        };

        return Ok(
            MapEntity {
                map_type: MapEntityType::Default {
                    om_terrain,
                    weight: 100,
                },
                fill,
                palettes,
                computed_parameters,
                tiles,
                size,
                terrain,
                furniture,
                items: Default::default(),
                place_nested: vec![],
            }
        );
    }
}