use std::path::Path;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::map::resources::MapEntity;

pub(crate) mod saver;
pub(crate) mod loader;

#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct Project {
    pub map_entity: MapEntity,
    pub map_save_path: Option<Box<Path>>,
}