use std::path::PathBuf;

use bevy::prelude::Resource;

#[derive(Debug, Resource)]
pub struct Settings {
    pub selected_cdda_dir: PathBuf,
    pub selectable_tilesets: Vec<String>,
    pub selected_tileset: Option<String>
}

impl Default for Settings {
    fn default() -> Self {
        return Self {
            selected_cdda_dir: "".into(),
            selectable_tilesets: vec![],
            selected_tileset: None,
        }
    }
}