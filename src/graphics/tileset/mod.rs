use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut};
use serde::{Deserialize, Serialize};

use crate::common::TileId;
use crate::project::loader::Load;

pub(crate) mod current;
pub(crate) mod legacy;

pub trait TilesetLoader<T>: Load<T> {
    fn get_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, Handle<Image>>, anyhow::Error>;
}


