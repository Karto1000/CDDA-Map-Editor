use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Error;
use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut, Vec2};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{DynamicImage, EncodableLayout, GenericImageView, ImageBuffer, imageops, Rgba};
use image::io::Reader;
use log::{debug, error, info, warn};
use serde::Deserialize;
use serde_json::Value;

use crate::common::{GetRandom, MeabyMulti, MeabyWeighted, TileId, Weighted};
use crate::common::io::{Load, LoadError};
use crate::graphics::{Corner, Edge, FullCardinal, Sprite, SpriteType};
use crate::graphics::tileset::{GetBackground, GetForeground};
use crate::graphics::tileset::TilesetLoader;

const TILESET_INFO_NAME: &'static str = "tileset.txt";
const AMOUNT_OF_SPRITES_PER_ROW: u32 = 16;

const FALLBACK_TILE_MAPPING: &'static [(&'static str, u32)] = &[
    // Ignore some textures at the start and end of each color
    ("!", 33),
    ("#", 35),
    ("$", 36),
    ("%", 37),
    ("&", 38),
    ("(", 40),
    (")", 41),
    ("*", 42),
    ("+", 43),
    ("0", 48),
    ("1", 49),
    ("2", 50),
    ("3", 51),
    ("4", 52),
    ("5", 53),
    ("6", 54),
    ("7", 55),
    ("8", 56),
    ("9", 57),
    (":", 58),
    (";", 59),
    ("<", 60),
    ("=", 61),
    ("?", 62),
    ("@", 63),
    ("A", 64),
    ("B", 65),
    ("C", 66),
    ("D", 67),
    ("E", 68),
    ("F", 69),
    ("G", 70),
    ("H", 71),
    ("I", 72),
    ("J", 73),
    ("K", 74),
    ("L", 75),
    ("M", 76),
    ("N", 77),
    ("O", 78),
    ("P", 79),
    ("Q", 80),
    ("R", 81),
    ("S", 82),
    ("T", 83),
    ("U", 84),
    ("V", 85),
    ("W", 86),
    ("X", 87),
    ("Y", 88),
    ("Z", 89),
    ("[", 90),
    (r"\", 91),
    ("]", 92),
    ("^", 93),
    ("_", 94),
    ("`", 95),
    ("{", 122),
    ("}", 124),
    ("|", 178)
];

#[derive(Debug)]
pub struct TilesetInfo {
    pub pixelscale: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}

#[derive(Debug, Deserialize)]
pub struct TileGroup {
    pub file: String,
    #[serde(rename = "//")]
    pub range: Option<String>,
    pub tiles: Vec<TilesetTileDescriptor>,
    pub ascii: Option<Vec<AsciiTilesetDescriptor>>,

    pub sprite_width: Option<u32>,
    pub sprite_height: Option<u32>,
    pub sprite_offset_x: Option<i32>,
    pub sprite_offset_y: Option<i32>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct AdditionalTile {
    id: String,
    fg: Option<MeabyMulti<MeabyWeighted<i32>>>,
    bg: Option<MeabyMulti<MeabyWeighted<i32>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TilesetTileDescriptor {
    pub id: MeabyMulti<String>,

    // a sprite root name that will be put on foreground
    pub fg: Option<MeabyMulti<MeabyWeighted<i32>>>,

    // another sprite root name that will be the background; can be empty for no background
    pub bg: Option<MeabyMulti<MeabyWeighted<i32>>>,

    #[serde(rename = "rotates")]
    pub is_rotate_allowed: Option<bool>,

    #[serde(rename = "multitile")]
    is_multitile: Option<bool>,

    #[serde(rename = "animated")]
    is_animated: Option<bool>,

    additional_tiles: Option<Vec<AdditionalTile>>,
}

#[derive(Debug, Deserialize)]
pub struct AsciiTilesetDescriptor {
    offset: u32,
    bold: bool,
    color: String,
}

#[derive(Debug)]
pub struct LegacyTileset {
    pub name: String,
    pub config_file_name: String,
    pub info: TilesetInfo,
    pub tiles: Vec<TileGroup>,
}

fn get_image_from_tileset(image: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Image {
    let tile_sprite = image.view(
        x,
        y,
        width,
        height,
    );

    let image = Image::new(
        Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        tile_sprite.to_image().to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );

    return image;
}

fn get_xy_from_index(index: &i32, last_group_index: i32) -> Vec2 {
    let local_tile_index: u32 = (index - last_group_index) as u32;

    return Vec2::new(
        (local_tile_index % AMOUNT_OF_SPRITES_PER_ROW) as f32,
        ((local_tile_index / AMOUNT_OF_SPRITES_PER_ROW) as f32).floor(),
    );
}

fn get_sprite_trait_from_single_fg(
    fg: &MeabyWeighted<i32>,
    loaded_sprites: &HashMap<i32, Handle<Image>>,
) -> Option<Arc<dyn GetForeground>> {
    let handle = match fg {
        MeabyWeighted::NotWeighted(fg) => {
            loaded_sprites.get(fg).unwrap()
        }
        MeabyWeighted::Weighted(w) => {
            loaded_sprites.get(&w.value).unwrap()
        }
    };

    return Some(Arc::new(SingleForeground { sprite: handle.clone() }));
}

fn get_sprite_trait_from_multi_fg(
    fg: &Vec<MeabyWeighted<i32>>,
    loaded_sprites: &HashMap<i32, Handle<Image>>,
) -> Option<Arc<dyn GetForeground>> {
    let mut textures: Vec<Weighted<Handle<Image>>> = Vec::new();

    for meaby_weighted in fg.iter() {
        match meaby_weighted {
            MeabyWeighted::NotWeighted(v) => {
                match loaded_sprites.get(v) {
                    None => {
                        warn!("Could not find sprite for bg {:?}", v)
                    }
                    Some(sprite) => {
                        textures.push(Weighted {
                            value: sprite.clone(),
                            // Give default weight of 1 to all sprites that are not weighted
                            // TODO: Revisit
                            // Check if this is actually correct
                            weight: 1,
                        })
                    }
                }
            }
            MeabyWeighted::Weighted(w) => {
                textures.push(Weighted::new(loaded_sprites.get(&w.value).unwrap().clone(), w.weight))
            }
        }
    }

    return Some(Arc::new(WeightedForeground { weighted_sprites: textures }));
}

fn get_sprite_trait_from_single_bg(
    bg: &MeabyWeighted<i32>,
    loaded_sprites: &HashMap<i32, Handle<Image>>,
) -> Option<Arc<dyn GetBackground>> {
    let handle = match bg {
        MeabyWeighted::NotWeighted(bg) => {
            match loaded_sprites.get(bg) {
                None => return None,
                Some(sprite) => sprite
            }
        }
        MeabyWeighted::Weighted(w) => {
            match loaded_sprites.get(&w.value) {
                None => return None,
                Some(sprite) => sprite
            }
        }
    };

    return Some(Arc::new(SingleBackground { sprite: handle.clone() }));
}

fn get_sprite_trait_from_multi_bg(
    bg: &Vec<MeabyWeighted<i32>>,
    loaded_sprites: &HashMap<i32, Handle<Image>>,
) -> Option<Arc<dyn GetBackground>> {
    let mut textures: Vec<Weighted<Handle<Image>>> = Vec::new();

    for meaby_weighted in bg.iter() {
        match meaby_weighted {
            MeabyWeighted::NotWeighted(v) => {
                match loaded_sprites.get(v) {
                    None => {
                        warn!("Could not find sprite for bg {:?}", v)
                    }
                    Some(sprite) => {
                        textures.push(Weighted {
                            value: sprite.clone(),
                            // Give default weight of 1 to all sprites that are not weighted
                            // TODO: Revisit
                            // Check if this is actually correct
                            weight: 1,
                        })
                    }
                }
            }
            MeabyWeighted::Weighted(w) => {
                match loaded_sprites.get(&w.value) {
                    None => {
                        warn!("Could not find sprite for bg {:?}", bg);
                    }
                    Some(sprite) => {
                        textures.push(Weighted {
                            value: sprite.clone(),
                            weight: w.weight,
                        });
                    }
                }
            }
        }
    }

    return Some(Arc::new(WeightedBackground { weighted_sprites: textures }));
}

fn get_single_fg_and_bg(
    loaded_sprites: &HashMap<i32, Handle<Image>>,
    fg: &MeabyMulti<MeabyWeighted<i32>>,
    bg: &Option<MeabyMulti<MeabyWeighted<i32>>>,
) -> (Option<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>) {
    let get_fg = match fg {
        MeabyMulti::Multi(multi) => {
            get_sprite_trait_from_multi_fg(
                multi,
                loaded_sprites,
            )
        }
        MeabyMulti::Single(fg) => {
            get_sprite_trait_from_single_fg(
                &fg,
                loaded_sprites,
            )
        }
    };

    let get_bg = match bg {
        Some(bg) => {
            match bg {
                MeabyMulti::Multi(multi) => {
                    get_sprite_trait_from_multi_bg(
                        multi,
                        loaded_sprites,
                    )
                }
                MeabyMulti::Single(bg) => {
                    get_sprite_trait_from_single_bg(
                        &bg,
                        loaded_sprites,
                    )
                }
            }
        }
        None => None
    };

    return (get_fg, get_bg);
}

fn get_multi_fg_and_bg(
    loaded_sprites: &HashMap<i32, Handle<Image>>,
    assets: &mut ResMut<Assets<Image>>,
    fg: &MeabyMulti<MeabyWeighted<i32>>,
    bg: &Option<MeabyMulti<MeabyWeighted<i32>>>,
) -> (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>) {
    let get_fg = match fg {
        MeabyMulti::Single(fg) => {
            // Seriously? Why would you not just put the same id in the list four times?
            // Ok, I am stupid, I just realized that this means the sprite is rotated
            let fg = match fg {
                MeabyWeighted::NotWeighted(fg) => fg.clone(),
                MeabyWeighted::Weighted(w) => w.value
            };

            let sprite = loaded_sprites.get(&fg).unwrap();

            let image = assets.get(sprite).unwrap();
            let mut rotated = Vec::new();

            let image_width = image.width();
            let image_height = image.height();

            let dyn_image = DynamicImage::from(ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(image_width, image_height, image.data.clone()).unwrap());

            for i in 0..4 {
                let new_image = match i {
                    0 => {
                        dyn_image.as_bytes().to_vec()
                    }
                    1 => {
                        imageops::rotate270(&dyn_image).to_vec()
                    }
                    2 => {
                        imageops::rotate180(&dyn_image).to_vec()
                    }
                    3 => {
                        imageops::rotate90(&dyn_image).to_vec()
                    }
                    _ => { panic!() }
                };

                let image = Image::new(
                    Extent3d {
                        width: 32,
                        height: 32,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    new_image,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::all(),
                );

                let data: Arc<dyn GetForeground> = Arc::new(SingleForeground { sprite: assets.add(image) });
                rotated.push(data);
            }

            vec![
                rotated.get(0).unwrap().clone(),
                rotated.get(1).unwrap().clone(),
                rotated.get(2).unwrap().clone(),
                rotated.get(3).unwrap().clone(),
            ]
        }
        MeabyMulti::Multi(v) => {
            // Direction is NW, SW, SE, NE
            let mut getters = Vec::new();

            for fg in v.iter() {
                let arc = get_sprite_trait_from_single_fg(
                    &fg,
                    loaded_sprites,
                );

                getters.push(arc.unwrap());
            }

            getters
        }
    };

    let get_bg = match bg {
        Some(bg) => {
            match bg {
                MeabyMulti::Multi(multi) => {
                    get_sprite_trait_from_multi_bg(
                        multi,
                        loaded_sprites,
                    )
                }
                MeabyMulti::Single(bg) => {
                    get_sprite_trait_from_single_bg(
                        bg,
                        loaded_sprites,
                    )
                }
            }
        }
        None => None
    };

    return (get_fg, get_bg);
}


pub struct LegacyTilesetLoader {
    pub path: PathBuf,
}

impl LegacyTilesetLoader {
    pub fn new(path: PathBuf) -> Self {
        return Self {
            path
        };
    }
}

pub struct WeightedForeground {
    weighted_sprites: Vec<Weighted<Handle<Image>>>,
}

impl GetForeground for WeightedForeground {
    fn get_sprite(&self) -> &Handle<Image> {
        return self.weighted_sprites.get_random_weighted().unwrap();
    }
}

pub struct SingleForeground {
    sprite: Handle<Image>,
}

impl SingleForeground {
    pub fn new(sprite: Handle<Image>) -> Self {
        return Self {
            sprite
        };
    }
}

impl GetForeground for SingleForeground {
    fn get_sprite(&self) -> &Handle<Image> {
        return &self.sprite;
    }
}


pub struct WeightedBackground {
    weighted_sprites: Vec<Weighted<Handle<Image>>>,
}

impl GetBackground for WeightedBackground {
    fn get_sprite(&self) -> &Handle<Image> {
        return self.weighted_sprites.get_random_weighted().unwrap();
    }
}

pub struct SingleBackground {
    sprite: Handle<Image>,
}

impl GetBackground for SingleBackground {
    fn get_sprite(&self) -> &Handle<Image> {
        return &self.sprite;
    }
}

impl Load<LegacyTileset> for LegacyTilesetLoader {
    fn load(&self) -> Result<LegacyTileset, LoadError> {
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

        return Ok(LegacyTileset {
            name: tileset_name.to_string(),
            config_file_name: config_file_name.to_string(),
            info: tileset_info,
            tiles,
        });
    }
}

impl TilesetLoader<LegacyTileset, i32> for LegacyTilesetLoader {
    fn load_textures(&self) -> Result<HashMap<i32, Image>, Error> {
        let tileset = self.load().unwrap();
        let mut textures: HashMap<i32, Image> = HashMap::new();
        let mut out_of_range = Vec::new();

        for group in tileset.tiles.iter() {
            if group.file == "fallback.png".to_string() {
                continue;
            }

            let image = Reader::open(self.path.join(PathBuf::from_str(group.file.as_str()).unwrap()))
                .unwrap()
                .decode()
                .unwrap();

            // TODO Revisit
            // Not a good way to do this, but i just couldn't for the life of me figure out how to get the range
            // Of the fg values of a tileset group without reading the comment
            let range_vec = group.range.as_ref().unwrap().split(" to ").collect::<Vec<&str>>();
            let mut start: u32 = range_vec.first().unwrap().split("range ").collect::<Vec<&str>>().last().unwrap().parse().unwrap();
            // -1 because the start is defined as 1
            start -= 1;

            let end: u32 = range_vec.last().unwrap().parse().unwrap();

            for tile in group.tiles.iter() {
                let group_width: u32 = group.sprite_width.unwrap_or(tileset.info.tile_width);
                let group_height: u32 = group.sprite_height.unwrap_or(tileset.info.tile_height);

                match &tile.fg {
                    None => {
                        info!("No fg for tile {:?}", tile.id);
                    }
                    Some(fg) => {
                        match fg {
                            MeabyMulti::Multi(multi) => {
                                for fg in multi.iter() {
                                    match fg {
                                        MeabyWeighted::NotWeighted(fg) => {
                                            if textures.contains_key(fg) {
                                                continue;
                                            }

                                            let xy = get_xy_from_index(fg, start as i32);

                                            let image = get_image_from_tileset(
                                                &image,
                                                xy.x as u32 * group_width,
                                                xy.y as u32 * group_height,
                                                group_width,
                                                group_height,
                                            );

                                            textures.insert(*fg, image);
                                        }
                                        MeabyWeighted::Weighted(w) => {
                                            if textures.contains_key(&w.value) {
                                                continue;
                                            }

                                            let xy = get_xy_from_index(&w.value, start as i32);

                                            let image = get_image_from_tileset(
                                                &image,
                                                xy.x as u32 * group_width,
                                                xy.y as u32 * group_height,
                                                group_width,
                                                group_height,
                                            );

                                            textures.insert(w.value, image);
                                        }
                                    }
                                }
                            }
                            MeabyMulti::Single(fg) => {
                                match fg {
                                    MeabyWeighted::NotWeighted(fg) => {
                                        if textures.contains_key(fg) {
                                            continue;
                                        }

                                        let xy = get_xy_from_index(fg, start as i32);

                                        if fg < &(start as i32) || fg > &(end as i32) {
                                            warn!("fg {} out of range", fg);
                                            out_of_range.push((fg, xy));
                                            continue;
                                        }

                                        println!("{:?} {} {}", xy, fg, start);

                                        let image = get_image_from_tileset(
                                            &image,
                                            xy.x as u32 * group_width,
                                            xy.y as u32 * group_height,
                                            group_width,
                                            group_height,
                                        );

                                        textures.insert(*fg, image);
                                    }
                                    MeabyWeighted::Weighted(w) => {
                                        if textures.contains_key(&w.value) {
                                            continue;
                                        }

                                        let xy = get_xy_from_index(&w.value, start as i32);

                                        if w.value < start as i32 || w.value > end as i32 {
                                            continue;
                                        }

                                        let image = get_image_from_tileset(
                                            &image,
                                            xy.x as u32 * group_width,
                                            xy.y as u32 * group_height,
                                            group_width,
                                            group_height,
                                        );

                                        textures.insert(w.value, image);
                                    }
                                }
                            }
                        }
                    }
                }

                match &tile.additional_tiles {
                    None => {}
                    Some(tiles) => {
                        for additional_tile in tiles {
                            match &additional_tile.fg {
                                None => {
                                    info!("Additional Tile with parent id {:?} has no fg", tile.fg);
                                }
                                Some(fg) => {
                                    match fg {
                                        MeabyMulti::Multi(multi) => {
                                            for fg in multi.iter() {
                                                match fg {
                                                    MeabyWeighted::NotWeighted(fg) => {
                                                        if textures.contains_key(fg) {
                                                            continue;
                                                        }

                                                        let xy = get_xy_from_index(fg, start as i32);

                                                        let image = get_image_from_tileset(
                                                            &image,
                                                            xy.x as u32 * group_width,
                                                            xy.y as u32 * group_height,
                                                            group_width,
                                                            group_height,
                                                        );

                                                        textures.insert(*fg, image);
                                                    }
                                                    MeabyWeighted::Weighted(w) => {
                                                        if textures.contains_key(&w.value) {
                                                            continue;
                                                        }

                                                        let xy = get_xy_from_index(&w.value, start as i32);

                                                        let image = get_image_from_tileset(
                                                            &image,
                                                            xy.x as u32 * group_width,
                                                            xy.y as u32 * group_height,
                                                            group_width,
                                                            group_height,
                                                        );

                                                        textures.insert(w.value, image);
                                                    }
                                                }
                                            }
                                        }
                                        MeabyMulti::Single(fg) => {
                                            match fg {
                                                MeabyWeighted::NotWeighted(fg) => {
                                                    if textures.contains_key(fg) {
                                                        continue;
                                                    }

                                                    let xy = get_xy_from_index(fg, start as i32);

                                                    // For some fucking reason the
                                                    // Grass tiles in the UndeadPeopleTileset specify a fg id which isn't
                                                    // even available in the file
                                                    // TODO FIX

                                                    if fg < &(start as i32) || fg > &(end as i32) {
                                                        continue;
                                                    }

                                                    let image = get_image_from_tileset(
                                                        &image,
                                                        xy.x as u32 * group_width,
                                                        xy.y as u32 * group_height,
                                                        group_width,
                                                        group_height,
                                                    );

                                                    textures.insert(*fg, image);
                                                }
                                                MeabyWeighted::Weighted(w) => {
                                                    if textures.contains_key(&w.value) {
                                                        continue;
                                                    }

                                                    let xy = get_xy_from_index(&w.value, start as i32);

                                                    if w.value < start as i32 || w.value > end as i32 {
                                                        continue;
                                                    }

                                                    let image = get_image_from_tileset(
                                                        &image,
                                                        xy.x as u32 * group_width,
                                                        xy.y as u32 * group_height,
                                                        group_width,
                                                        group_height,
                                                    );

                                                    textures.insert(w.value, image);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        return Ok(textures);
    }
    fn load_fallback_textures(&self) -> Result<HashMap<String, Image>, Error> {
        let tileset = self.load().unwrap();
        let mut fallback_textures: HashMap<String, Image> = HashMap::new();

        for group in tileset.tiles.iter() {
            if group.file != "fallback.png".to_string() { continue; }

            let image = Reader::open(self.path.join(PathBuf::from_str(group.file.as_str()).unwrap()))
                .unwrap()
                .decode()
                .unwrap();

            for color in group.ascii.as_ref().unwrap().iter() {
                for (character, index) in FALLBACK_TILE_MAPPING {
                    let x = (color.offset + 1 + index) % AMOUNT_OF_SPRITES_PER_ROW * tileset.info.tile_width;
                    let y = (color.offset + 1 + index) / AMOUNT_OF_SPRITES_PER_ROW * tileset.info.tile_height;

                    let fallback_image = get_image_from_tileset(
                        &image,
                        x,
                        y,
                        tileset.info.tile_width,
                        tileset.info.tile_height,
                    );

                    fallback_textures.insert(format!("{}_{}", character, color.color), fallback_image);
                }
            };
        }

        return Ok(fallback_textures);
    }

    fn load_sprite_handles(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, SpriteType>, Error> {
        let tileset = self.load().unwrap();
        let mut loaded_sprites: HashMap<i32, Handle<Image>> = HashMap::new();

        self.load_textures()
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, image_resource.add(v)))
            .for_each(|(k, v)| { loaded_sprites.insert(k, v); });

        let mut assigned_textures: HashMap<TileId, SpriteType> = HashMap::new();

        for group in tileset.tiles.iter() {
            let offset_x = group.sprite_offset_x.unwrap_or(0);
            let offset_y = group.sprite_offset_y.unwrap_or(0);

            for tile in group.tiles.iter() {
                let get_main_fg: Option<Arc<dyn GetForeground>> = match &tile.fg {
                    None => { continue; }
                    Some(fg) => {
                        match fg {
                            MeabyMulti::Single(fg) => {
                                match fg {
                                    MeabyWeighted::NotWeighted(fg) => {
                                        match loaded_sprites.get(fg) {
                                            None => {
                                                error!("No Sprite found for fg {}", fg);
                                                None
                                            }
                                            Some(sprite) => {
                                                Some(Arc::new(SingleForeground { sprite: sprite.clone() }))
                                            }
                                        }
                                    }
                                    MeabyWeighted::Weighted(w) => {
                                        match loaded_sprites.get(&w.value) {
                                            None => None,
                                            Some(sprite) => Some(Arc::new(SingleForeground { sprite: sprite.clone() }))
                                        }
                                    }
                                }
                            }
                            MeabyMulti::Multi(fg) => {
                                let mut sprites: Vec<Weighted<Handle<Image>>> = Vec::new();

                                for fg in fg.iter() {
                                    match fg {
                                        MeabyWeighted::NotWeighted(fg) => {
                                            // TODO: Check how to handle weights
                                            sprites.push(Weighted::new(loaded_sprites.get(fg).unwrap().clone(), 0));
                                        }
                                        MeabyWeighted::Weighted(w) => {
                                            sprites.push(Weighted::new(loaded_sprites.get(&w.value).unwrap().clone(), w.weight));
                                        }
                                    }
                                }

                                Some(Arc::new(WeightedForeground { weighted_sprites: sprites }))
                            }
                        }
                    }
                };

                let get_main_bg: Option<Arc<dyn GetBackground>> = match &tile.bg {
                    None => None,
                    Some(bg) => {
                        match bg {
                            MeabyMulti::Single(bg) => {
                                match bg {
                                    MeabyWeighted::NotWeighted(bg) => {
                                        match loaded_sprites.get(bg) {
                                            None => None,
                                            Some(sprite) => Some(Arc::new(SingleBackground { sprite: sprite.clone() }))
                                        }
                                    }
                                    MeabyWeighted::Weighted(w) => {
                                        // TODO: Revisit
                                        // Not sure what to do here
                                        match loaded_sprites.get(&w.value) {
                                            None => None,
                                            Some(sprite) => Some(Arc::new(SingleBackground { sprite: sprite.clone() }))
                                        }
                                    }
                                }
                            }
                            MeabyMulti::Multi(bg) => {
                                // Handle weird case when bg is an empty array
                                if bg.len() > 0 {
                                    let mut sprites: Vec<Weighted<Handle<Image>>> = Vec::new();

                                    for bg in bg.iter() {
                                        match bg {
                                            MeabyWeighted::NotWeighted(bg) => {
                                                // TODO: Revisit
                                                // Not sure what to do here
                                                match loaded_sprites.get(bg) {
                                                    None => {}
                                                    Some(sprite) => sprites.push(Weighted::new(sprite.clone(), 0))
                                                };
                                            }
                                            MeabyWeighted::Weighted(w) => {
                                                match loaded_sprites.get(&w.value) {
                                                    None => {}
                                                    Some(sprite) => sprites.push(Weighted::new(sprite.clone(), w.weight))
                                                };
                                            }
                                        }
                                    }

                                    Some(Arc::new(WeightedBackground { weighted_sprites: sprites }))
                                } else {
                                    None
                                }
                            }
                        }
                    }
                };

                match &tile.additional_tiles {
                    None => {
                        match &tile.id {
                            MeabyMulti::Single(v) => {
                                debug!("Loaded tile {:?}", v);
                                assigned_textures.insert(
                                    v.clone(),
                                    SpriteType::Single(Sprite {
                                        fg: get_main_fg.clone(),
                                        bg: get_main_bg.clone(),
                                        offset_x,
                                        offset_y,
                                        is_animated: tile.is_animated.unwrap_or(false),
                                    }),
                                );
                            }
                            MeabyMulti::Multi(v) => {
                                for value in v.iter() {
                                    debug!("Loaded tile {:?}", value);
                                    assigned_textures.insert(
                                        value.clone(),
                                        SpriteType::Single(Sprite {
                                            fg: get_main_fg.clone(),
                                            bg: get_main_bg.clone(),
                                            offset_x,
                                            offset_y,
                                            is_animated: tile.is_animated.unwrap_or(false),
                                        }),
                                    );
                                }
                            }
                        };
                    }
                    Some(additional_tiles) => {
                        let ids = match &tile.id {
                            MeabyMulti::Multi(multi) => multi.clone(),
                            MeabyMulti::Single(id) => vec![id.clone()]
                        };

                        for id in ids {
                            let mut center: Option<Sprite> = None;
                            let mut corner: Option<Corner> = None;
                            let mut t_connection: Option<FullCardinal> = None;
                            let mut edge: Option<Edge> = None;
                            let mut end_piece: Option<FullCardinal> = None;
                            let mut unconnected: Option<Sprite> = None;

                            for additional_tile in additional_tiles.iter() {
                                let fg = match additional_tile.fg.as_ref() {
                                    Some(fg) => fg,
                                    None => { continue; }
                                };

                                let bg = &additional_tile.bg;

                                // TODO: Figure out what a id of 'broken' means
                                match additional_tile.id.as_str() {
                                    "center" => {
                                        let v = get_single_fg_and_bg(
                                            &loaded_sprites,
                                            fg,
                                            bg,
                                        );

                                        center = Some(Sprite {
                                            fg: v.0,
                                            bg: v.1,
                                            offset_x,
                                            offset_y,
                                            is_animated: tile.is_animated.unwrap_or(false),
                                        })
                                    }
                                    "corner" => {
                                        let v = get_multi_fg_and_bg(
                                            &loaded_sprites,
                                            image_resource,
                                            fg,
                                            bg,
                                        );
                                        corner = Some(Corner::from((v.0, v.1, offset_x, offset_y, tile.is_animated.unwrap_or(false)
                                        )))
                                    }
                                    "t_connection" => {
                                        let v = get_multi_fg_and_bg(
                                            &loaded_sprites,
                                            image_resource,
                                            fg,
                                            bg,
                                        );
                                        t_connection = Some(FullCardinal::from((v.0, v.1, offset_x, offset_y, tile.is_animated.unwrap_or(false))));
                                    }
                                    "edge" => {
                                        let v = get_multi_fg_and_bg(
                                            &loaded_sprites,
                                            image_resource,
                                            fg,
                                            bg,
                                        );
                                        edge = Some(Edge::from((v.0, v.1, offset_x, offset_y, tile.is_animated.unwrap_or(false))));
                                    }
                                    "end_piece" => {
                                        let v = get_multi_fg_and_bg(
                                            &loaded_sprites,
                                            image_resource,
                                            fg,
                                            bg,
                                        );
                                        end_piece = Some(FullCardinal::from((v.0, v.1, offset_x, offset_y, tile.is_animated.unwrap_or(false))));
                                    }
                                    "unconnected" => {
                                        let (get_fg, get_bg) = get_single_fg_and_bg(
                                            &loaded_sprites,
                                            fg,
                                            bg,
                                        );
                                        unconnected = Some(Sprite {
                                            fg: get_fg,
                                            bg: get_bg,
                                            offset_x,
                                            offset_y,
                                            is_animated: tile.is_animated.unwrap_or(false),
                                        });
                                    }
                                    _ => { warn!("Got Unexpected id {} for fg {:?}", additional_tile.id, fg) }
                                }
                            }

                            let default_sprite = Sprite {
                                fg: get_main_fg.clone(),
                                bg: get_main_bg.clone(),
                                offset_x: 0,
                                offset_y: 0,
                                is_animated: false,
                            };

                            assigned_textures.insert(
                                id.clone(),
                                SpriteType::Multitile {
                                    center: center.unwrap_or(default_sprite.clone()),
                                    corner: corner.unwrap_or(Corner {
                                        north_west: default_sprite.clone(),
                                        south_west: default_sprite.clone(),
                                        south_east: default_sprite.clone(),
                                        north_east: default_sprite.clone(),
                                    }),
                                    t_connection: t_connection.unwrap_or(FullCardinal {
                                        north: default_sprite.clone(),
                                        east: default_sprite.clone(),
                                        south: default_sprite.clone(),
                                        west: default_sprite.clone(),
                                    }),
                                    edge: edge.unwrap_or(Edge {
                                        north_south: default_sprite.clone(),
                                        east_west: default_sprite.clone(),
                                    }),
                                    end_piece: end_piece.unwrap_or(FullCardinal {
                                        north: default_sprite.clone(),
                                        east: default_sprite.clone(),
                                        south: default_sprite.clone(),
                                        west: default_sprite.clone(),
                                    }),
                                    unconnected: unconnected.unwrap_or(default_sprite.clone()),
                                },
                            );
                        }
                    }
                }
            }
        };

        return Ok(assigned_textures);
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;

    use image::io::Reader;

    use crate::common::io::Load;
    use crate::graphics::tileset::legacy::{get_image_from_tileset, LegacyTilesetLoader};
    use crate::graphics::tileset::TilesetLoader;

    #[test]
    pub fn test_load_legacy_tileset() {
        let data = LegacyTilesetLoader::new(PathBuf::from("./testing_data")).load().unwrap();

        assert_eq!(data.config_file_name, "tile_config.json");
        assert_eq!(data.name, "TEST");
        assert_eq!(data.info.tile_width, 32);
        assert_eq!(data.info.tile_height, 32);
        assert_eq!(data.info.pixelscale, 1);

        let first_group = data.tiles.first().unwrap();

        assert_eq!(first_group.file, "normal_items.png");
        assert_eq!(first_group.range.as_ref().unwrap(), "range 1 to 2");

        let first_tile = first_group.tiles.first().unwrap().clone();

        assert_eq!(first_tile.id.single(), "10mm_fmj".to_string());
        assert_eq!(first_tile.fg.unwrap().single().not_weighted(), 1);
        assert_eq!(first_tile.bg.unwrap().single().not_weighted(), 9443);
        assert_eq!(first_tile.is_rotate_allowed, Some(false));
    }

    #[test]
    pub fn test_load_textures() {
        let data = LegacyTilesetLoader::new(PathBuf::from("./testing_data")).load_textures().unwrap();

        let image = Reader::open(PathBuf::from("./testing_data/normal_items.png"))
            .unwrap()
            .decode()
            .unwrap();

        let item = data.get(&1).unwrap();

        let supposed_data = get_image_from_tileset(
            &image,
            32,
            0,
            32,
            32,
        );

        assert_eq!(item.data, supposed_data.data);
    }
}

