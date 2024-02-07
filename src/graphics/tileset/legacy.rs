use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut};
use either::Either;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::io::Reader;

use serde_json::Value;
use crate::common::{MeabyWeighted, TileId};
use crate::graphics::tileset::{Tileset, TilesetLoader};

use crate::project::loader::{Load, LoadError};

const TILESET_INFO_NAME: &'static str = "tileset.txt";

#[derive(Debug)]
pub struct TilesetInfo {
    pub pixelscale: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

#[derive(Debug, Deserialize)]
pub struct TileGroup {
    pub file: String,
    pub tiles: Vec<TilesetTileDescriptor>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Multiline {
    No,
    Yes {
        #[serde(rename = "multiline")]
        is_multiline: Option<bool>,
        additional_tiles: Option<Vec<Box<TilesetTileDescriptor>>>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MeabyMulti<T> {
    Single(T),
    Multi(Vec<T>),
}

#[derive(Debug, Deserialize)]
pub struct TilesetTileDescriptor {
    pub id: MeabyMulti<String>,

    // a sprite root name that will be put on foreground
    // TODO: Figure out why UndeadPeopleTileset uses numbers instead of Strings
    pub fg: Option<MeabyMulti<MeabyWeighted<i32>>>,

    // another sprite root name that will be the background; can be empty for no background
    pub bg: Option<MeabyMulti<MeabyWeighted<i32>>>,

    #[serde(rename = "rotates")]
    pub is_rotate_allowed: Option<bool>,
    pub multiline: Option<Multiline>,
}

#[derive(Debug)]
pub struct LegacyTileset {
    pub name: String,
    pub config_file_name: String,
    pub info: TilesetInfo,
    pub tiles: Vec<TileGroup>,
}

pub struct LegacyTilesetLoader {
    pub path: PathBuf,
}

impl LegacyTilesetLoader {
    pub fn new(path: &PathBuf) -> Self {
        return Self {
            path: path.clone()
        };
    }
}

impl Load<Tileset> for LegacyTilesetLoader {
    fn load(&self) -> Result<Tileset, LoadError> {
        let info_file = File::open(self.path.join(PathBuf::from_str(TILESET_INFO_NAME).unwrap())).unwrap();

        let mut buf_reader = BufReader::new(info_file);
        let mut line_content = String::new();

        let mut iteration = 0;

        let mut tileset_name = String::new();
        let mut config_file_name = String::new();

        while let Ok(bytes_read) = buf_reader.read_line(&mut line_content) {
            if bytes_read == 0 { break; }

            match iteration {
                // Name of Tileset
                0 => {
                    tileset_name = line_content.split("NAME: ")
                        .last()
                        .unwrap_or("Unknown Name")
                        .to_string()
                        .replace("\n", "");
                }
                // Config file name
                2 => {
                    config_file_name = line_content
                        .split("JSON: ")
                        .last()
                        .unwrap()
                        .to_string()
                        .replace("\n", "");
                }
                _ => {}
            }

            line_content = String::new();
            iteration += 1;
        }

        let config_file_path_buf = PathBuf::from_str(config_file_name.as_str()).unwrap();
        let tileset_file_value = serde_json::from_str::<Value>(&*fs::read_to_string(self.path.join(config_file_path_buf)).unwrap()).unwrap();
        let tileset_object = tileset_file_value.as_object().unwrap();

        let tile_info = tileset_object
            .get("tile_info")
            .unwrap()
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_object()
            .unwrap();

        let tileset_info = TilesetInfo {
            pixelscale: tile_info.get("pixelscale").unwrap().as_u64().unwrap() as u32,
            tile_width: tile_info.get("width").unwrap().as_u64().unwrap() as u32,
            tile_height: tile_info.get("height").unwrap().as_u64().unwrap() as u32,
        };

        let tiles_new = tileset_object
            .get("tiles-new")
            .unwrap()
            .as_array()
            .unwrap();

        let mut tiles = Vec::new();

        for tile_group in tiles_new.iter() {
            tiles.push(serde_json::from_value::<TileGroup>(tile_group.clone()).unwrap());
        }

        return Ok(Tileset::Legacy(LegacyTileset {
            name: tileset_name.to_string(),
            config_file_name: config_file_name.to_string(),
            info: tileset_info,
            tiles,
        }));
    }
}

impl TilesetLoader for LegacyTilesetLoader {
    fn get_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, Handle<Image>>, anyhow::Error> {
        let tileset = match self.load().unwrap() {
            Tileset::Legacy(t) => t,
            _ => { panic!("LegacyTilesetLoader must return LegacyTileSet") }
        };

        let mut textures = HashMap::new();

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

        textures.insert(TileId { 0: "t_grass".into() }, image_resource.add(texture));
        return Ok(textures);
    }
}