use std::collections::HashMap;
use std::path::PathBuf;

use bevy::math::Vec2;

use bevy::tasks::futures_lite::StreamExt;
use log::{info};
use serde::{Deserialize, Serialize};
use serde_json::{Value};

use crate::common::{Coordinates, MeabyWeighted, TileId};
use crate::common::io::{Load, LoadError};
use crate::editor_data::CDDAData;
use crate::map::resources::ComputedParameters;
use crate::map::resources::MapEntity;
use crate::map::resources::MapEntityType;
use crate::palettes::{MeabyParam, MapGenValue, MapObjectId, PaletteId, ParameterType};
use crate::tiles::components::Tile;

pub type ParameterId = String;

pub struct MapEntityLoader<'a> {
    pub path: PathBuf,
    pub id: String,
    pub cdda_data: &'a CDDAData
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub default: MapGenValue,
}

fn compute_palettes(
    cdda_data: &CDDAData,
    parameters: &HashMap<String, String>, 
    palettes: &Vec<MapObjectId<MeabyParam>>
) -> HashMap<PaletteId, ComputedParameters> {
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
                            MeabyParam::TileId(i) => {
                                i.clone()
                            }
                            MeabyParam::Parameter(_) => { todo!() }
                        }
                    }
                    MeabyWeighted::Weighted(_) => { todo!() }
                }
            }
        };

        let associated_palette = cdda_data.palettes.get(&palette_id).unwrap();

        let mut this = HashMap::new();

        for (name, parameter) in associated_palette.parameters.iter() {
            this.insert(name.clone(), parameter.default.get_value());
        }

        let computed_palette_parameters = ComputedParameters {
            this: this.clone(),
            palettes: compute_palettes(cdda_data, &this, &associated_palette.palettes),
        };

        computed_palettes.insert(palette_id, computed_palette_parameters.clone());

        info!("Computed Parameters for {:?} parameters: {:?}", palette, computed_palette_parameters)
    }

    return computed_palettes;
}

impl<'a> Load<MapEntity> for MapEntityLoader<'a> {
    fn load(&self) -> Result<MapEntity, LoadError> {
        let mut map_type: Option<MapEntityType> = None;

        let objects = serde_json::from_str::<Vec<HashMap<String, Value>>>(std::fs::read_to_string(&self.path).unwrap().as_str()).unwrap();
        let mut om_based_size = Vec2::new(24., 24.);

        let mapgen_entity = objects
            .into_iter()
            .find(|o| {
                return match o.get("om_terrain") {
                    None => false,
                    Some(s) => match serde_json::from_value::<MapObjectId<String>>(s.clone()) {
                        Ok(id) => {
                            match id {
                                MapObjectId::Grouped(group) => {
                                    let ids: Vec<String> = group.iter().map(|mw| mw.value().clone()).collect();

                                    let any_matches = ids.iter().any(|id| *id == self.id);

                                    if any_matches {
                                        map_type = Some(
                                            MapEntityType::Multi {
                                                om_terrain: ids,
                                                weight: 100,
                                            }
                                        )
                                    }

                                    any_matches
                                }
                                MapObjectId::Nested(nested) => {
                                    // Each om_terrain id in a nested vec equals to 24 tiles
                                    // Each nested vec in the parent vec equals to 24 tiles
                                    om_based_size.x = 24. * nested.first().unwrap().len() as f32;
                                    om_based_size.y = 24. * nested.len() as f32;

                                    let any_matches = nested.iter().flatten().map(|mw| mw.value()).any(|v| v.clone() == self.id);

                                    if any_matches {
                                        map_type = Some(
                                            MapEntityType::Nested {
                                                om_terrain: nested.iter()
                                                    .map(|v| v.iter()
                                                        .map(|mw| mw.value().clone()).collect())
                                                    .collect(),
                                                weight: 100,
                                            }
                                        )
                                    }

                                    any_matches
                                }
                                MapObjectId::Single(id) => {
                                    let any_matches = id.value().clone() == self.id;

                                    if any_matches {
                                        map_type = Some(
                                            MapEntityType::Default {
                                                om_terrain: id.value().clone(),
                                                weight: 100,
                                            }
                                        )
                                    }

                                    any_matches
                                }
                                _ => todo!()
                            }
                        }
                        Err(_) => { todo!() }
                    }
                };
            })
            .unwrap();

        let object = mapgen_entity.get("object").unwrap();

        let parameters = match object.get("parameters") {
            None => HashMap::new(),
            Some(v) => serde_json::from_value::<HashMap<ParameterId, Parameter>>(v.clone()).unwrap()
        };

        let rows: Vec<String> = serde_json::from_value(object.get("rows").unwrap().clone()).unwrap();

        let mut tiles = HashMap::new();

        let size = match object.get("mapgensize") {
            None => om_based_size,
            Some(v) => match v {
                Value::Array(a) => Vec2::new(
                    a.first().unwrap().as_f64().unwrap() as f32,
                    a.last().unwrap().as_f64().unwrap() as f32,
                ),
                _ => panic!()
            }
        };
        info!("Loaded Map Object Size: {}", size);

        for (row, tile) in rows.iter().enumerate() {
            // to_string returns quotes so we use as_str
            for (column, char) in tile.as_str().chars().enumerate() {
                tiles.insert(
                    Coordinates::new(column as i32, row as i32),
                    Tile::from(char),
                );
            }
        }

        let palettes: Vec<MapObjectId<MeabyParam>> = serde_json::from_value(object.get("palettes").unwrap_or(&Value::Array(Vec::new())).clone()).unwrap();

        let mut this = HashMap::new();

        for (parameter_id, parameter) in parameters.iter() {
            this.insert(
                parameter_id.clone(),
                parameter.default.get_value(),
            );
        }

        let computed_parameters = ComputedParameters {
            this: this.clone(),
            palettes: compute_palettes(self.cdda_data, &this, &palettes),
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
            Some(v) => Some(String::from(v.as_str().unwrap().to_string()))
        };

        return Ok(
            MapEntity {
                map_type: map_type.unwrap(),
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
