use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use bevy::math::Vec2;
use bevy::prelude::{EventWriter, Resource};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::map::{TilePlaceEvent, Tiles};

#[derive(Serialize, Deserialize, Debug, Resource, Clone, Default)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub tiles: Tiles,
}

impl MapEntity {
    pub fn new(name: String, size: Vec2) -> Self {
        return Self {
            name,
            weight: 100,
            tiles: Tiles { tiles: HashMap::new(), size },
        };
    }

    pub fn export(&self) -> Result<Value, anyhow::Error> {
        return Ok(serde_json::json!([{
            "method": "json",
            "om_terrain": self.name,
            "type": "mapgen",
            "weight": 100,
            "object": {
                "fill_ter": "t_floor",
                "rows": self.tiles.export()?,
                "palettes": [ "domestic_general_and_variant_palette" ],
            }
        }]));
    }

    pub fn load(&mut self, e_set_tile: &mut EventWriter<TilePlaceEvent>, entity: &MapEntity) {
        self.name = entity.name.clone();
        self.weight = entity.weight;

        self.tiles.load(e_set_tile, &entity.tiles);
    }
}