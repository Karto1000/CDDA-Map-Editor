use std::collections::HashMap;
use std::error::Error;

use bevy::asset::Handle;
use bevy::prelude::{Commands, default, Entity, Image, IVec2, Res, Resource, SpriteBundle, Transform, Vec2, Vec3};
use serde::{Deserializer, Serialize};
use serde_json::Value;

use crate::grid::Grid;
use crate::tiles::{Tile, TileType};

pub(crate) mod system;

#[derive(Serialize, Debug, Resource)]
pub struct Tiles {
    pub size: Vec2,
    pub tiles: HashMap<(i32, i32), Tile>,

    #[serde(skip)]
    pub texture: Handle<Image>,
}

impl Tiles {
    pub fn json(&self) -> Result<Value, Box<dyn Error>> {
        let mut rows: Vec<String> = vec![];

        for _ in 0..self.size.y as i32 {
            let mut row = String::with_capacity(self.size.x as usize);
            (0..self.size.x as i32).for_each(|i| {
                row.insert(i as usize, " ".parse::<char>().unwrap())
            });
            rows.push(row);
        };

        return Ok(Value::Array(rows.iter().map(|e| Value::String(e.clone())).collect()));
    }

    pub fn set_tile_at(
        &mut self,
        commands: &mut Commands,
        cords: (i32, i32),
        tile_type: TileType,
        res_grid: &Res<Grid>,
    ) -> Option<Entity> {
        if self.tiles.get(&(cords.0, cords.1)).is_some() { return None; }

        let tile = Tile { tile_type, x: cords.0, y: cords.1 };

        let c = commands.spawn((
            tile,
            SpriteBundle {
                texture: self.texture.clone(),
                transform: Transform {
                    translation: Vec3 {
                        // Spawn off screen
                        x: -1000.0,
                        y: -1000.0,
                        z: 0.0,
                    },
                    scale: Vec3 {
                        x: res_grid.tile_size / res_grid.default_tile_size,
                        y: res_grid.tile_size / res_grid.default_tile_size,
                        z: 0.,
                    },
                    ..default()
                },
                ..default()
            },
        ));

        self.tiles.insert(
            cords,
            tile,
        );

        return Some(c.id());
    }

    pub fn get_size(&self) -> Option<IVec2> {
        let mut keys_sorted_x: Vec<(i32, i32)> = self.tiles.clone().into_keys().collect();
        let mut keys_sorted_y: Vec<(i32, i32)> = self.tiles.clone().into_keys().collect();

        keys_sorted_x.sort_by(|(x1, _), (x2, _)| x1.cmp(x2));
        keys_sorted_y.sort_by(|(_, y1), (_, y2)| y1.cmp(y2));

        let leftmost_tile = keys_sorted_x.first().cloned().unwrap();
        let rightmost_tile = keys_sorted_x.last().cloned().unwrap();

        let topmost_tile = keys_sorted_y.first().cloned().unwrap();
        let bottommost_tile = keys_sorted_y.last().cloned().unwrap();

        return Some(IVec2::new((rightmost_tile.0 - leftmost_tile.0).abs() + 1, (bottommost_tile.1 - topmost_tile.1).abs() + 1));
    }
}

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