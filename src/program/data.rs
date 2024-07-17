use bevy::prelude::{Color, Component, Resource, States};
use bevy_egui::egui::Color32;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use num::ToPrimitive;
use crate::common::io::Load;

use crate::palettes::data::Palette;
use crate::palettes::io::PalettesLoader;
use crate::project::data::{Project, ProjectSaveState};
use crate::ui::style::Style;

#[derive(Default, States, Clone, Hash, Debug, Eq, PartialEq)]
pub enum ProgramState {
    ProjectOpen,
    #[default]
    NoneOpen,
}

#[derive(Component, Debug)]
pub struct OpenedProject {
    pub index: usize
}

#[derive(Resource)]
pub struct Program {
    pub history: Vec<ProjectSaveState>,
    pub config: Config,
    pub menus: Menus,
    pub projects: Vec<Project>,
}

impl Program {
    pub fn new(projects: Vec<Project>, history: Vec<ProjectSaveState>) -> Self {
        return Self {
            projects,
            history,
            config: Config::default(),
            menus: Menus::default(),
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
    pub is_create_project_menu_open: bool
}

