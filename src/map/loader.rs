use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

use bevy::math::Vec2;

use bevy::tasks::futures_lite::StreamExt;
use log::{info};
use serde::{Deserialize, Serialize};
use serde_json::{Value};

use crate::common::{Coordinates, MeabyWeighted, TileId};
use crate::common::io::{Load, LoadError};
use crate::editor_data::CDDAData;
use crate::map::resources::{ComputedParameters, Multi, Nested, Single, TileSelection};
use crate::map::resources::MapEntity;
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
            MapObjectId::Single(mp) => {
                match mp {
                    MeabyParam::TileId(i) => {
                        i.clone()
                    }
                    MeabyParam::Parameter(_) => { todo!() }
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

impl Load<Single> for MapEntityLoader<'_> {
    fn load(&self) -> Result<Single, LoadError> {
        let objects = serde_json::from_str::<Vec<HashMap<String, Value>>>(std::fs::read_to_string(&self.path).unwrap().as_str()).unwrap();

        let mapgen_entity = objects
            .into_iter()
            .find(|o| {
                return match o.get("om_terrain") {
                    None => false,
                    Some(s) => match serde_json::from_value::<MapObjectId<String>>(s.clone()) {
                        Ok(id) => {
                            match id {
                                MapObjectId::Single(id) => {
                                    let any_matches = id.clone() == self.id;
                                    any_matches
                                }
                                _ => false
                            }
                        }
                        Err(_) => { todo!() }
                    }
                };
            })
            .unwrap();

        let om_terrain = mapgen_entity.get("om_terrain").unwrap();
        let object = mapgen_entity.get("object").unwrap();
        let rows: Vec<String> = serde_json::from_value(object.get("rows").unwrap().clone()).unwrap();
        let parameters = match object.get("parameters") {
            None => HashMap::new(),
            Some(v) => serde_json::from_value::<HashMap<ParameterId, Parameter>>(v.clone()).unwrap()
        };
        let palettes: Vec<MapObjectId<MeabyParam>> = serde_json::from_value(object.get("palettes").unwrap_or(&Value::Array(Vec::new())).clone()).unwrap();

        let mut tiles = HashMap::new();

        for (row, tile) in rows.iter().enumerate() {
            // to_string returns quotes so we use as_str
            for (column, char) in tile.as_str().chars().enumerate() {
                tiles.insert(
                    Coordinates::new(column as i32, row as i32),
                    Tile::from(char),
                );
            }
        }

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

        info!("Loaded Single Mapgen Object {}", om_terrain);

        return Ok(
            Single {
                om_terrain: om_terrain.to_string(),
                tile_selection: TileSelection {
                    fill_ter: fill,
                    computed_parameters,
                    palettes,
                    terrain,
                    furniture,
                },
                tiles,
            }
        );
    }
}

impl Load<Multi> for MapEntityLoader<'_> {
    fn load(&self) -> Result<Multi, LoadError> {
        todo!()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CDDAMapgenObject {
    fill_ter: Option<String>,
    rows: Vec<String>,
    palettes: Vec<MapObjectId<MeabyParam>>,

    terrain: Option<HashMap<char, MapObjectId<MeabyWeighted<MeabyParam>>>>,
    furniture: Option<HashMap<char, MapObjectId<MeabyWeighted<MeabyParam>>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CDDANestedMapgenObject {
    om_terrain: Vec<Vec<String>>,
    method: String,
    #[serde(rename = "type")]
    om_type: String,
    parameters: Option<HashMap<ParameterId, Parameter>>,
    object: CDDAMapgenObject,
}

impl Load<Nested> for MapEntityLoader<'_> {
    fn load(&self) -> Result<Nested, LoadError> {
        let objects: Vec<CDDANestedMapgenObject> = serde_json::from_str::<Vec<Value>>(read_to_string(&self.path).unwrap().as_str())
            .unwrap()
            .into_iter()
            .filter_map(|hm| {
                return match serde_json::from_value(hm) {
                    Err(e) => None,
                    Ok(v) => Some(v)
                };
            })
            .collect();

        // TODO: Handle
        let entity = objects.first().unwrap();

        let mut tiles = HashMap::new();

        for (row, tile) in entity.object.rows.iter().enumerate() {
            // to_string returns quotes, so we use as_str
            for (column, char) in tile.as_str().chars().enumerate() {
                tiles.insert(
                    Coordinates::new(column as i32, row as i32),
                    Tile::from(char),
                );
            }
        }

        let mut this = HashMap::new();

        let terrain = entity.object.terrain.clone().unwrap_or(HashMap::new());
        let furniture = entity.object.furniture.clone().unwrap_or(HashMap::new());
        let parameters = entity.parameters.clone().unwrap_or(HashMap::new());

        for (parameter_id, parameter) in parameters.iter() {
            this.insert(
                parameter_id.clone(),
                parameter.default.get_value(),
            );
        }

        let computed_parameters = ComputedParameters {
            this: this.clone(),
            palettes: compute_palettes(self.cdda_data, &this, &entity.object.palettes),
        };

        info!("Loaded Nested Om Mapgen Object {:?}", entity.om_terrain);

        return Ok(
            Nested {
                row_size: entity.om_terrain.get(0).unwrap().len(),
                om_terrain: entity.om_terrain.iter().flatten().map(|s| s.clone()).collect(),
                tile_selection: TileSelection {
                    fill_ter: entity.object.fill_ter.clone(),
                    computed_parameters,
                    palettes: entity.object.palettes.clone(),
                    terrain,
                    furniture,
                },
                tiles,
            }
        );
    }
}