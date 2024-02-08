use std::path::PathBuf;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::map::resources::MapEntity;

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct Project {
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