use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::{Res, Vec2};
use log::{info, warn};
use serde_json::Value;

use crate::common::Coordinates;
use crate::common::io::{Load, LoadError};
use crate::EditorData;
use crate::map::resources::{MapEntity, MapEntityType};
use crate::palettes::Palette;
use crate::tiles::components::Tile;

pub struct MapEntityImporter<'a> {
    path: PathBuf,
    id: String,
    all_palettes: &'a HashMap<String, Palette>
}

impl<'a> MapEntityImporter<'a> {
    pub fn new(path: PathBuf, id: String, all_palettes: &'a HashMap<String, Palette>) -> Self {
        return Self {
            path,
            id,
            all_palettes
        };
    }
}

impl<'a> Load<MapEntity> for MapEntityImporter<'a> {
    fn load(&self) -> Result<MapEntity, LoadError> {
        let parsed = serde_json::from_str::<Value>(fs::read_to_string(&self.path).unwrap().as_str())
            .unwrap();

        let found_object = match parsed
            .as_array()
            .unwrap()
            .iter()
            .find(|v| {
                match v.get("nested_mapgen_id") {
                    None => false,
                    Some(v) => v.as_str().unwrap().to_string() == self.id
                }
            }) {
            None => {
                parsed.as_array().unwrap().iter().find(|v| {
                    match v.get("om_terrain") {
                        None => false,
                        Some(v) => v.as_str().unwrap().to_string() == self.id
                    }
                }).unwrap()
            }
            Some(v) => v
        };

        let object = found_object.get("object").unwrap();

        let json_tiles = object
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap();

        let map_type: MapEntityType = serde_json::from_value(found_object.clone()).unwrap();
        let mut tiles = HashMap::new();
        let palettes: Vec<String> = serde_json::from_value(object.get("palettes").unwrap().clone()).unwrap();

        let size = Vec2::new(
            json_tiles.get(0).unwrap().as_str().unwrap().len() as f32,
            json_tiles.len() as f32,
        );

        for (row, tile) in json_tiles.iter().enumerate() {
            // to_string returns quotes so we use as_str
            for (column, char) in tile.as_str().unwrap().chars().enumerate() {
                tiles.insert(
                    Coordinates::new(column as i32, row as i32),
                    Tile::from(char),
                );
            }
        }

        let mut map_entity = MapEntity {
            map_type,
            tiles,
            size,
            place_nested: Vec::new(),
            palettes: vec![],
            terrain: serde_json::from_value(object.get("terrain").unwrap().clone()).unwrap(),
            furniture: HashMap::new(),
            items: HashMap::new(),
            parameters: HashMap::new(),
        };

        for palette in palettes {
            let string_palette = palette.to_string();

            let palette = match self.all_palettes.get(&string_palette) {
                None => {
                    warn!("Could not find Palette {} specified in {}", string_palette, self.id);
                    continue;
                }
                Some(v) => v
            };

            info!("Successfully loaded Palette {}", palette.id);

            map_entity.add_palette(palette);
        }

        return Ok(map_entity);
    }
}