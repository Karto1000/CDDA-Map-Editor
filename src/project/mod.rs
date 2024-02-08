use bevy::prelude::{Event, Resource};
use serde::{Deserialize, Serialize};

use crate::project::loader::Load;
use crate::project::saver::Save;

pub(crate) mod saver;
pub(crate) mod loader;
pub(crate) mod resources;


