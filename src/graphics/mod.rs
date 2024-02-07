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
    pub fn load(tileset_loader: impl TilesetLoader, mut image_resource: ResMut<Assets<Image>>) -> Self {
        // Use the grass texture as a placeholder
        // let mut textures: HashMap<TileId, Handle<Image>> = HashMap::new();
        let tileset = tileset_loader.get_textures(&mut image_resource).unwrap();

        // let grass = Reader::open("assets/grass.png").unwrap().decode().unwrap().as_bytes().to_vec();
        //
        // let texture = Image::new(
        //     Extent3d {
        //         width: 32,
        //         height: 32,
        //         depth_or_array_layers: 1,
        //     },
        //     TextureDimension::D2,
        //     grass,
        //     TextureFormat::Rgba8UnormSrgb,
        // );
        //
        // textures.insert(TileId { 0: "t_grass".into() }, image_resource.add(texture));

        return Self {
            textures: tileset
        };
    }

    pub fn get_texture(&self, tile_id: &TileId) -> &Handle<Image> {
        return self.textures.get(tile_id).unwrap();
    }
}