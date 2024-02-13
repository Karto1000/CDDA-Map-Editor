use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use bevy::prelude::Vec2;
use serde_json::Value;

use crate::common::Coordinates;
use crate::common::io::{Load, LoadError};
use crate::map::resources::{MapEntity, MapEntityType};
use crate::palettes::Palette;
use crate::tiles::components::Tile;

pub struct MapEntityImporter {
    path: PathBuf,
    nested_mapgen_id: String,
}

impl MapEntityImporter {
    pub fn new(path: PathBuf, nested_mapgen_id: String) -> Self {
        return Self {
            path,
            nested_mapgen_id,
        };
    }
}

impl Load<MapEntity> for MapEntityImporter {
    fn load(&self) -> Result<MapEntity, LoadError> {
        let parsed = serde_json::from_str::<Value>(fs::read_to_string(&self.path).unwrap().as_str())
            .unwrap();

        let found_object = parsed
            .as_array()
            .unwrap()
            .iter()
            .find(|v| {
                return v.get("nested_mapgen_id").unwrap().as_str().unwrap().to_string() == self.nested_mapgen_id;
            })
            .unwrap();

        let object = found_object.get("object").unwrap();

        let json_tiles = object
            .get("rows")
            .unwrap()
            .as_array()
            .unwrap();

        let map_type: MapEntityType = serde_json::from_value(found_object.clone()).unwrap();
        let mut tiles = HashMap::new();
        // let palettes: Vec<Palette> = serde_json::from_value(object.get("palettes").unwrap().clone()).unwrap();

        let size = Vec2::new(
            json_tiles.get(0).unwrap().to_string().len() as f32,
            json_tiles.len() as f32,
        );

        for (row, tile) in json_tiles.iter().enumerate() {
            for (column, char) in tile.to_string().chars().enumerate() {
                tiles.insert(
                    Coordinates::new(column as i32, row as i32),
                    Tile::from(char),
                );
            }
        }

        return Ok(MapEntity {
            map_type,
            tiles,
            size,
            place_nested: None,
            // TODO Load Palette
            palettes: vec![],
        });
    }
}