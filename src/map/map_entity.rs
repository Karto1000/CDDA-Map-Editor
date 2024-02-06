use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::{EventWriter, Resource};
use serde::{Deserialize, Serialize};

use crate::map::{Coordinates, TilePlaceEvent};
use crate::tiles::Tile;

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub tiles: HashMap<Coordinates, Tile>,
    pub size: Vec2,
}

impl Default for MapEntity {
    fn default() -> Self {
        return Self {
            name: "unnamed".into(),
            weight: 0,
            tiles: HashMap::new(),
            size: Vec2::new(24., 24.),
        };
    }
}

impl MapEntity {
    pub fn new(name: String, size: Vec2) -> Self {
        return Self {
            name,
            weight: 100,
            tiles: HashMap::new(),
            size,
        };
    }

    pub fn spawn(&mut self, e_set_tile: &mut EventWriter<TilePlaceEvent>) {
        for cords in self.tiles.keys().into_iter() {
            let tile = self.tiles.get(cords).unwrap();
            e_set_tile.send(TilePlaceEvent { tile: *tile })
        }
    }

    pub fn set_tile_at(
        &mut self,
        character: char,
        cords: (i32, i32),
        e_set_tile: &mut EventWriter<TilePlaceEvent>,
    ) {
        let coordinates = Coordinates { x: cords.0, y: cords.1 };
        if self.tiles.get(&coordinates).is_some() { return; }

        let tile = Tile { character, x: cords.0, y: cords.1 };

        e_set_tile.send(TilePlaceEvent { tile });

        self.tiles.insert(
            coordinates,
            tile,
        );
    }
}