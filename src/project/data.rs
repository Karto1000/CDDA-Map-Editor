use std::path::PathBuf;

use bevy::prelude::{Event, Resource};
use serde::{Deserialize, Serialize};

use crate::map::data::MapEntity;

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub map_entity: MapEntity,
    pub save_state: ProjectSaveState,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectSaveState {
    /// If the Project was saved somewhere
    /// Contains the path to the saved project
    Saved(PathBuf),

    /// If the project was auto saved
    /// Contains the path to an auto save file
    AutoSaved(PathBuf),

    /// If the project was not saved
    #[default] NotSaved,
}

#[derive(Event)]
pub struct OpenProjectAtIndex {
    pub index: u32,
}

#[derive(Event)]
pub struct CreateProject {
    pub project: Project
}

#[derive(Event)]
pub struct CloseProject {}


