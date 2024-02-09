use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut};
use serde::{Deserialize, Serialize};

use crate::common::TileId;
use crate::graphics::SpriteType;
use crate::project::loader::Load;

pub(crate) mod current;
pub(crate) mod legacy;

pub trait TilesetLoader<T, Id>: Load<T> {
    fn load_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<Id, Handle<Image>>, anyhow::Error>;
    fn assign_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, SpriteType>, anyhow::Error>;
}


