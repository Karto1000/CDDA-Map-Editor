use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::{GetRandom, TileId, Weighted};

pub(crate) mod loader;

type RegionId = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct TerrainAndFurniture {
    // Example "t_region_groundcover": { "t_grass": 12000, "t_grass_dead": 2000, "t_dirt": 1000 },
    pub terrain: HashMap<RegionId, HashMap<TileId, u32>>,

    // Example "f_region_forest_water": { "f_burdock": 4, "f_purple_loosestrife": 3, "f_japanese_knotweed": 2, "f_lily": 1 },
    pub furniture: HashMap<RegionId, HashMap<TileId, u32>>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RegionSettings {
    pub id: String,

    // We only really care about these three fields

    pub default_oter: Vec<String>,
    pub default_groundcover: Vec<Weighted<String>>,

    pub region_terrain_and_furniture: TerrainAndFurniture,
}

impl RegionSettings {
    pub fn get_random_terrain_from_region(&self, region_id: &RegionId) -> Option<&TileId> {
        if let Some(terrain) = self.region_terrain_and_furniture.terrain.get(region_id) {
            let picked = terrain.get_random_weighted().unwrap();

            // Regions can have region ids themselves
            return match self.get_random_terrain_from_region(&picked) {
                Some(t) => Some(t),
                None => Some(picked)
            };
        }

        return None;
    }

    pub fn get_random_furniture_from_region(&self, region_id: &RegionId) -> Option<&TileId> {
        if let Some(furniture) = self.region_terrain_and_furniture.furniture.get(region_id) {
            let picked = furniture.get_random_weighted().unwrap();

            // Regions can have region ids themselves
            return match self.get_random_furniture_from_region(&picked) {
                Some(t) => Some(t),
                None => Some(picked)
            };
        }

        return None;
    }
}