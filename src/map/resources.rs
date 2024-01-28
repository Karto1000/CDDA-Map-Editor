use std::collections::HashMap;
use std::error::Error;
use bevy::asset::Handle;
use bevy::math::Vec2;
use bevy::prelude::{Image, Resource};
use serde::Serialize;
use serde_json::Value;
use crate::map::Tiles;

#[derive(Serialize, Debug, Resource)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub map: Tiles,
}

impl MapEntity {
    pub fn new(name: String, size: Vec2, texture: Handle<Image>) -> Self {
        return Self {
            name,
            weight: 100,
            map: Tiles { tiles: HashMap::new(), texture, size },
        };
    }

    pub fn json(&self) -> Result<Value, Box<dyn Error>> {
        return Ok(serde_json::json!([{
            "method": "json",
            "om_terrain": self.name,
            "type": "mapgen",
            "weight": 100,
            "object": {
                "fill_ter": "t_floor",
                "rows": self.map.json().unwrap(),
                "palettes": [ "domestic_general_and_variant_palette" ],
            }
        }]));
    }
}