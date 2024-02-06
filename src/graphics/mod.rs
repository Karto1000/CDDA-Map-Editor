use std::collections::HashMap;

use bevy::prelude::{Assets, Handle, Image, ResMut, Resource};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::io::Reader;

#[derive(Resource)]
pub struct GraphicsResource {
    pub textures: HashMap<char, Handle<Image>>,
}

impl GraphicsResource {
    pub fn load(mut image_resource: ResMut<Assets<Image>>) -> Self {
        // Use the grass texture as a placeholder
        let mut textures: HashMap<char, Handle<Image>> = HashMap::new();

        let grass = Reader::open("assets/grass.png").unwrap().decode().unwrap().as_bytes().to_vec();

        let texture = Image::new(
            Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            grass,
            TextureFormat::Rgba8UnormSrgb,
        );

        textures.insert('g', image_resource.add(texture));

        return Self {
            textures
        };
    }

    pub fn get_texture(&self, character: &char) -> &Handle<Image> {
        return self.textures.get(character).unwrap();
    }
}