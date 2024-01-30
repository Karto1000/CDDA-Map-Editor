use std::collections::HashMap;
use std::error::Error;
use anyhow::anyhow;
use bevy::asset::Handle;
use bevy::math::Vec2;
use bevy::prelude::{Commands, Image, IVec2, Res, Resource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::grid::resources::Grid;
use crate::map::Tiles;

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub tiles: Tiles,
}

impl MapEntity {
    pub fn new(name: String, size: Vec2, texture: Handle<Image>) -> Self {
        return Self {
            name,
            weight: 100,
            tiles: Tiles { tiles: HashMap::new(), texture, size },
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

    pub fn load(&mut self, commands: &mut Commands, res_grid: &Res<Grid>, entity: &MapEntity) {
        self.name = entity.name.clone();
        self.weight = entity.weight;

        self.tiles.load(commands, res_grid, &entity.tiles);
    }
}