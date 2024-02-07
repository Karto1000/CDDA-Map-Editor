use std::collections::HashMap;

use bevy::prelude::{Assets, Handle, Image, ResMut, Resource};

use crate::common::TileId;
use crate::graphics::tileset::TilesetLoader;
use crate::project::loader::Load;

pub(crate) mod tileset;

#[derive(Resource)]
pub struct GraphicsResource {
    pub textures: HashMap<TileId, Handle<Image>>,
}

impl GraphicsResource {
    pub fn load<T>(tileset_loader: impl TilesetLoader<T>, mut image_resource: ResMut<Assets<Image>>) -> Self {
        let tileset = tileset_loader.get_textures(&mut image_resource).unwrap();

        return Self {
            textures: tileset
        };
    }

    pub fn get_texture(&self, tile_id: &TileId) -> &Handle<Image> {
        return self.textures.get(tile_id).unwrap();
    }
}