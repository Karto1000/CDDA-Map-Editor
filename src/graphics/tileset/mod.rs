use std::collections::HashMap;
use std::io::BufRead;
use std::str::FromStr;

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut};
use serde::{Deserialize, Serialize};

use crate::common::io::Load;
use crate::common::TileId;
use crate::graphics::SpriteType;

pub(crate) mod current;
pub(crate) mod legacy;

pub trait GetForeground: Send + Sync {
    fn get_sprite(&self) -> &Handle<Image>;
}

pub trait GetBackground: Send + Sync {
    fn get_sprite(&self) -> &Handle<Image>;
}

pub trait TilesetLoader<T, Id>: Load<T> {
    fn load_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<Id, Handle<Image>>, anyhow::Error>;
    fn load_fallback_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<String, Handle<Image>>, anyhow::Error>;
    fn assign_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, SpriteType>, anyhow::Error>;
}


