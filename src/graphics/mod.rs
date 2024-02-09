use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::{Assets, Handle, Image, ResMut, Resource};
use bevy_egui::egui::load::TextureLoader;

use crate::common::{Coordinates, TileId};
use crate::graphics::tileset::legacy::LegacyTileset;
use crate::graphics::tileset::TilesetLoader;
use crate::project::loader::Load;
use crate::project::resources::Project;

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

pub trait GetTexture: Send + Sync {
    fn get_texture(&self, project: &Project, character: &char, coordinates: &Coordinates) -> &Handle<Image>;
}

pub struct LegacyTextures {
    textures: HashMap<TileId, SpriteType>,
}

impl LegacyTextures {
    pub fn new(loader: impl TilesetLoader<LegacyTileset>, image_resource: &mut ResMut<Assets<Image>>) -> Self {
        let textures = loader.get_textures(image_resource).unwrap();

        return Self {
            textures
        };
    }
}


impl GetTexture for LegacyTextures {
    fn get_texture(&self, project: &Project, character: &char, coordinates: &Coordinates) -> &Handle<Image> {
        let sprite_type = self.textures.get(&project.map_entity.get_tile_id_from_character(character)).unwrap();

        return match sprite_type {
            SpriteType::Single(s) => s,
            SpriteType::Multitile { center, corner, t_connection, edge, end_piece, unconnected } => {
                let tiles_around = project.map_entity.get_tiles_around(coordinates);

                let is_tile_ontop_same_type = match tiles_around.get(0).unwrap().0 {
                    None => false,
                    Some(top) => top.character == *character
                };

                let is_tile_right_same_type = match tiles_around.get(1).unwrap().0 {
                    None => false,
                    Some(right) => right.character == *character
                };

                let is_tile_below_same_type = match tiles_around.get(2).unwrap().0 {
                    None => false,
                    Some(below) => below.character == *character
                };

                let is_tile_left_same_type = match tiles_around.get(3).unwrap().0 {
                    None => false,
                    Some(left) => left.character == *character
                };

                return match (is_tile_ontop_same_type, is_tile_right_same_type, is_tile_below_same_type, is_tile_left_same_type) {
                    // Some of the worst code i've ever written lol
                    (true, true, true, true) => &center,
                    (true, true, true, false) => &t_connection.west,
                    (true, true, false, true) => &t_connection.south,
                    (true, false, true, true) => &t_connection.east,
                    (false, true, true, true) => &t_connection.north,
                    (true, true, false, false) => &corner.south_west,
                    (true, false, false, true) => &corner.south_east,
                    (false, true, true, false) => &corner.north_west,
                    (false, false, true, true) => &corner.north_east,
                    (true, false, false, false) => &end_piece.south,
                    (false, true, false, false) => &end_piece.west,
                    (false, false, true, false) => &end_piece.north,
                    (false, false, false, true) => &end_piece.east,
                    (false, true, false, true) => &edge.east_west,
                    (true, false, true, false) => &edge.north_south,
                    (false, false, false, false) => &unconnected
                };
            }
        };
    }
}

#[derive(Resource)]
pub struct GraphicsResource {
    pub textures: Box<dyn GetTexture>,
}

impl GraphicsResource {
    pub fn new(tileset_loader: Box<dyn GetTexture>) -> Self {
        return Self {
            textures: tileset_loader
        };
    }
}