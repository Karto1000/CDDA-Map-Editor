use std::path::PathBuf;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Resource, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub selected_cdda_dir: Option<PathBuf>,
    pub selectable_tilesets: Vec<String>,
    pub selected_tileset: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        return Self {
            selected_cdda_dir: None,
            selectable_tilesets: vec![],
            selected_tileset: None,
        };
    }
}

impl Settings {
    pub fn data_json_dir(&self) -> Option<PathBuf> {
        return match &self.selected_cdda_dir {
            None => None,
            Some(dir) => Some(dir.join(r"data\json"))
        };
    }

    pub fn gfx_dir(&self) -> Option<PathBuf> {
        return match &self.selected_cdda_dir {
            None => None,
            Some(dir) => Some(dir.join(r"gfx"))
        };
    }
}
