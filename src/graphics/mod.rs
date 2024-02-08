use std::collections::HashMap;

use bevy::prelude::{Assets, Handle, Image, ResMut, Resource};

use crate::common::TileId;
use crate::graphics::tileset::TilesetLoader;
use crate::project::loader::Load;

pub(crate) mod tileset;

pub struct FullCardinal {
    pub north: Handle<Image>,
    pub east: Handle<Image>,
    pub south: Handle<Image>,
    pub west: Handle<Image>,
}

pub struct Corner {
    pub north_west: Handle<Image>,
    pub south_west: Handle<Image>,
    pub south_east: Handle<Image>,
    pub north_east: Handle<Image>,
}

pub struct Edge {
    pub north_south: Handle<Image>,
    pub east_west: Handle<Image>,
}

pub enum SpriteType {
    Single(Handle<Image>),
    Multitile {
        center: Handle<Image>,
        corner: Corner,
        t_connection: FullCardinal,
        edge: Edge,
        end_piece: FullCardinal,
        unconnected: Handle<Image>,
    },
}

#[derive(Resource)]
pub struct GraphicsResource {
    pub textures: HashMap<TileId, SpriteType>,
}

impl GraphicsResource {
    pub fn load<T>(tileset_loader: impl TilesetLoader<T>, mut image_resource: ResMut<Assets<Image>>) -> Self {
        let tileset = tileset_loader.get_textures(&mut image_resource).unwrap();

        return Self {
            textures: tileset
        };
    }

    pub fn get_texture(&self, tile_id: &TileId) -> &SpriteType {
        // TODO Add actual sensible default
        return self.textures.get(tile_id).unwrap();
    }
}