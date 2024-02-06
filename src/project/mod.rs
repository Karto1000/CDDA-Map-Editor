use std::path::PathBuf;

use bevy::prelude::{Event, Resource};
use serde::{Deserialize, Serialize};
use crate::map::map_entity::MapEntity;

use crate::project::loader::Load;
use crate::project::saver::Save;

pub(crate) mod saver;
pub(crate) mod loader;

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct Project {
    pub map_entity: MapEntity,
    pub save_state: ProjectSaveState,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
