pub(crate) mod io;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};
use crate::common::io::Load;
use crate::palettes::loader::PalettesLoader;
use crate::palettes::Palette;
use crate::project::resources::{Project, ProjectSaveState};

#[derive(Debug, Resource)]
pub struct EditorData {
    pub current_project_index: u32,
    pub projects: Vec<Project>,
    pub history: Vec<ProjectSaveState>,
    pub config: Config,
}

impl EditorData {
    pub fn get_current_project(&self) -> Option<&Project> {
        return self.projects.get(self.current_project_index as usize);
    }

    pub fn get_current_project_mut(&mut self) -> Option<&mut Project> {
        return self.projects.get_mut(self.current_project_index as usize);
    }
}

impl Default for EditorData {
    fn default() -> Self {
        return Self {
            current_project_index: 0,
            projects: vec![],
            history: vec![],
            config: Config::default(),
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CDDAData {
    pub path: PathBuf,
    pub palettes: HashMap<String, Palette>,
}

#[derive(Debug)]
pub struct Config {
    pub cdda_data: Option<Arc<CDDAData>>,
}

impl Config {
    pub fn load_cdda_dir(&mut self, cdda_dir: PathBuf) {
        let palettes_folder = PathBuf::from(format!("{}/data/json/mapgen_palettes", cdda_dir.to_str().unwrap())); 
        let palettes = PalettesLoader::new(palettes_folder).load().unwrap();

        self.cdda_data = Some(Arc::new(CDDAData {
            palettes,
            path: cdda_dir,
        }));
    }
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            cdda_data: Default::default()
        };
    }
}