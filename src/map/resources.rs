use std::collections::HashMap;

use bevy::math::Vec2;
use bevy::prelude::{EventWriter, Resource};
use serde::{Deserialize, Serialize};

use crate::map::{TilePlaceEvent, Tiles};

#[derive(Serialize, Deserialize, Debug, Resource, Clone)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub tiles: Tiles,
}

impl Default for MapEntity {
    fn default() -> Self {
        return Self {
            name: "unnamed".into(),
            weight: 0,
            tiles: Tiles::default(),
        };
    }
}

impl MapEntity {
    pub fn new(name: String, size: Vec2) -> Self {
        return Self {
            name,
            weight: 100,
            tiles: Tiles { tiles: HashMap::new(), size },
        };
    }

    #[inline]
    pub fn spawn(&mut self, e_set_tile: &mut EventWriter<TilePlaceEvent>) {
        self.tiles.spawn(e_set_tile)
    }
}