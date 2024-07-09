use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::{Color, Resource};
use bevy_egui::egui::Color32;
use num::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::common::io::Load;
use crate::palettes::loader::PalettesLoader;
use crate::palettes::Palette;
use crate::project::resources::{Project, ProjectSaveState};
use crate::ui::style::Style;

pub(crate) mod io;

#[derive(Debug, Resource)]
pub struct EditorData {
    pub current_project_index: Option<u32>,
    pub projects: Vec<Project>,
    pub history: Vec<ProjectSaveState>,
    pub config: Config,
    pub menus: Menus,
}

impl EditorData {
    pub fn get_current_project(&self) -> Option<&Project> {
        if self.current_project_index == None { return None; }
        return self.projects.get(self.current_project_index.unwrap() as usize);
    }

    pub fn get_current_project_mut(&mut self) -> Option<&mut Project> {
        if self.current_project_index == None { return None; }
        return self.projects.get_mut(self.current_project_index.unwrap() as usize);
    }
}

impl Default for EditorData {
    fn default() -> Self {
        return Self {
            current_project_index: None,
            projects: vec![],
            history: vec![],
            config: Config::default(),
            menus: Menus {
                is_settings_menu_open: false
            },
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CDDAData {
    pub palettes: HashMap<String, Palette>,
}

#[derive(Debug)]
pub struct Config {
    pub cdda_data: Option<Arc<CDDAData>>,
    pub style: Style,
}

impl Config {
    pub fn load_cdda_dir(&mut self, cdda_dir: PathBuf) {
        let palettes_folder = PathBuf::from(format!("{}/data/json/mapgen_palettes", cdda_dir.to_str().unwrap()));
        let palettes = PalettesLoader::new(palettes_folder).load().unwrap();

        self.cdda_data = Some(Arc::new(CDDAData {
            palettes,
        }));
    }
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            cdda_data: Default::default(),
            style: Style::dark(),
        };
    }
}

pub trait IntoColor32 {
    fn into_color32(self) -> Color32;
}

impl IntoColor32 for Color {
    fn into_color32(self) -> Color32 {
        return Color32::from_rgb(
            (self.r() * 255.).to_u8().unwrap(),
            (self.g() * 255.).to_u8().unwrap(),
            (self.b() * 255.).to_u8().unwrap(),
        );
    }
}

#[derive(Debug, Default)]
pub struct Menus {
    pub is_settings_menu_open: bool,
}