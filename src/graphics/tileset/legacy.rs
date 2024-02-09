use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut, Vec2};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{DynamicImage, GenericImageView};
use image::io::Reader;
use log::{debug, warn};
use rand::Rng;
use serde::Deserialize;
use serde_json::Value;

use crate::common::{MeabyWeighted, TileId, Weighted};
use crate::graphics::{Corner, Edge, FullCardinal, Sprite, SpriteType};
use crate::graphics::tileset::TilesetLoader;
use crate::project::loader::{Load, LoadError};

const TILESET_INFO_NAME: &'static str = "tileset.txt";
const AMOUNT_OF_SPRITES_PER_ROW: u32 = 16;

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
pub enum MeabyMulti<T> {
    Multi(Vec<T>),
    Single(T),
}

#[derive(Debug, Deserialize)]
pub struct AdditionalTile {
    id: String,
    fg: Option<MeabyMulti<MeabyWeighted<i32>>>,
    bg: Option<MeabyMulti<MeabyWeighted<i32>>>,
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

    #[serde(rename = "multitile")]
    is_multitile: Option<bool>,
    additional_tiles: Option<Vec<AdditionalTile>>,
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
    pub fn new(path: PathBuf) -> Self {
        return Self {
            path
        };
    }
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
    );

    return image;
}

fn get_xy_from_index(fg: &i32, last_group_index: i32) -> Vec2 {
    let local_tile_index: u32 = (fg - last_group_index) as u32;

    return Vec2::new(
        (local_tile_index % AMOUNT_OF_SPRITES_PER_ROW) as f32,
        ((local_tile_index / AMOUNT_OF_SPRITES_PER_ROW) as f32).floor(),
    );
}

pub trait GetForeground: Send + Sync {
    fn get_sprite(&self) -> &Handle<Image>;
}

pub struct WeightedForeground {
    weighted_sprites: Vec<Weighted<Handle<Image>>>,
}

impl GetForeground for WeightedForeground {
    fn get_sprite(&self) -> &Handle<Image> {
        let mut rng = rand::thread_rng();
        let random_index: usize = rng.gen_range(0..self.weighted_sprites.len());
        // TODO Take weights into account
        let random_sprite = self.weighted_sprites.get(random_index).unwrap();
        return &random_sprite.value;
    }
}

pub struct SingleForeground {
    sprite: Handle<Image>,
}

impl GetForeground for SingleForeground {
    fn get_sprite(&self) -> &Handle<Image> {
        return &self.sprite;
    }
}

pub trait GetBackground: Send + Sync {
    fn get_sprite(&self) -> &Handle<Image>;
}

pub struct WeightedBackground {
    weighted_sprites: Vec<Weighted<Handle<Image>>>,
}

impl GetBackground for WeightedBackground {
    fn get_sprite(&self) -> &Handle<Image> {
        todo!()
    }
}

pub struct SingleBackground {
    sprite: Handle<Image>,
}

impl GetBackground for SingleBackground {
    fn get_sprite(&self) -> &Handle<Image> {
        todo!()
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


fn get_sprite_trait_from_single_fg(
    fg: &MeabyWeighted<i32>,
    last_group_index: i32,
    image: &DynamicImage,
    image_resource: &mut ResMut<Assets<Image>>,
    tileset: &LegacyTileset,
) -> Option<Arc<dyn GetForeground>> {
    let handle = match fg {
        MeabyWeighted::NotWeighted(fg) => {
            let xy = get_xy_from_index(fg, last_group_index);

            // TODO: Figure out what to do if the sprite is inside another file
            if fg < &last_group_index {
                return None;
            }

            let image = get_image_from_tileset(
                &image,
                xy.x as u32 * tileset.info.tile_width,
                xy.y as u32 * tileset.info.tile_height,
                tileset.info.tile_width,
                tileset.info.tile_height,
            );

            image_resource.add(image)
        }
        MeabyWeighted::Weighted(w) => {
            let xy = get_xy_from_index(&w.value, last_group_index);

            // TODO: Figure out what to do if the sprite is inside another file
            if &w.value < &last_group_index {
                return None;
            }

            let image = get_image_from_tileset(
                &image,
                xy.x as u32 * tileset.info.tile_width,
                xy.y as u32 * tileset.info.tile_height,
                tileset.info.tile_width,
                tileset.info.tile_height,
            );

            image_resource.add(image)
        }
    };

    return Some(Arc::new(SingleForeground { sprite: handle }));
}

fn get_sprite_trait_from_multi_fg(
    fg: &Vec<MeabyWeighted<i32>>,
    last_group_index: i32,
    image: &DynamicImage,
    image_resource: &mut ResMut<Assets<Image>>,
    tileset: &LegacyTileset,
) -> Option<Arc<dyn GetForeground>> {
    let mut textures: Vec<Weighted<Handle<Image>>> = Vec::new();

    for meaby_weighted in fg.iter() {
        match meaby_weighted {
            MeabyWeighted::NotWeighted(v) => {
                // TODO: Figure out what to do here
                warn!("Elements with fg {:?} should be weighted", fg)
            }
            MeabyWeighted::Weighted(w) => {
                let xy = get_xy_from_index(&w.value, last_group_index);

                // TODO: Figure out what to do if the sprite is inside another file
                if w.value < last_group_index {
                    return None;
                }

                let image = get_image_from_tileset(
                    &image,
                    xy.x as u32 * tileset.info.tile_width,
                    xy.y as u32 * tileset.info.tile_height,
                    tileset.info.tile_width,
                    tileset.info.tile_height,
                );

                textures.push(Weighted { value: image_resource.add(image), weight: w.weight });
            }
        }
    }

    return Some(Arc::new(WeightedForeground { weighted_sprites: textures }));
}

fn get_sprite_trait_from_single_bg(
    bg: &MeabyWeighted<i32>,
    last_group_index: i32,
    image: &DynamicImage,
    image_resource: &mut ResMut<Assets<Image>>,
    tileset: &LegacyTileset,
) -> Option<Arc<dyn GetBackground>> {
    let handle = match bg {
        MeabyWeighted::NotWeighted(bg) => {
            let xy = get_xy_from_index(bg, last_group_index);

            // TODO: Figure out what to do if the sprite is inside another file
            if bg < &last_group_index {
                return None;
            }

            let image = get_image_from_tileset(
                &image,
                xy.x as u32 * tileset.info.tile_width,
                xy.y as u32 * tileset.info.tile_height,
                tileset.info.tile_width,
                tileset.info.tile_height,
            );

            image_resource.add(image)
        }
        MeabyWeighted::Weighted(w) => {
            let xy = get_xy_from_index(&w.value, last_group_index);

            // TODO: Figure out what to do if the sprite is inside another file
            if &w.value < &last_group_index {
                return None;
            }

            let image = get_image_from_tileset(
                &image,
                xy.x as u32 * tileset.info.tile_width,
                xy.y as u32 * tileset.info.tile_height,
                tileset.info.tile_width,
                tileset.info.tile_height,
            );

            image_resource.add(image)
        }
    };

    return Some(Arc::new(SingleBackground { sprite: handle }));
}

fn get_sprite_trait_from_multi_bg(
    bg: &Vec<MeabyWeighted<i32>>,
    last_group_index: i32,
    image: &DynamicImage,
    image_resource: &mut ResMut<Assets<Image>>,
    tileset: &LegacyTileset,
) -> Option<Arc<dyn GetBackground>> {
    let mut textures: Vec<Weighted<Handle<Image>>> = Vec::new();

    for meaby_weighted in bg.iter() {
        match meaby_weighted {
            MeabyWeighted::NotWeighted(v) => {
                // TODO: Figure out what to do here
                warn!("Elements with bg {:?} should be weighted", bg)
            }
            MeabyWeighted::Weighted(w) => {
                let xy = get_xy_from_index(&w.value, last_group_index);

                // TODO: Figure out what to do if the sprite is inside another file
                if w.value < last_group_index {
                    return None;
                }

                let image = get_image_from_tileset(
                    &image,
                    xy.x as u32 * tileset.info.tile_width,
                    xy.y as u32 * tileset.info.tile_height,
                    tileset.info.tile_width,
                    tileset.info.tile_height,
                );

                textures.push(Weighted { value: image_resource.add(image), weight: w.weight });
            }
        }
    }

    return Some(Arc::new(WeightedBackground { weighted_sprites: textures }));
}

fn get_single_fg_and_bg(
    image_resource: &mut ResMut<Assets<Image>>,
    fg: &MeabyMulti<MeabyWeighted<i32>>,
    bg: &Option<MeabyMulti<MeabyWeighted<i32>>>,
    last_group_index: i32,
    image: &DynamicImage,
    tileset: &LegacyTileset,
) -> (Option<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>) {
    let get_fg = match fg {
        MeabyMulti::Multi(multi) => {
            get_sprite_trait_from_multi_fg(
                multi,
                last_group_index,
                &image,
                image_resource,
                &tileset,
            )
        }
        MeabyMulti::Single(fg) => {
            get_sprite_trait_from_single_fg(
                &fg,
                last_group_index,
                &image,
                image_resource,
                &tileset,
            )
        }
    };

    let get_bg = match bg {
        Some(bg) => {
            match bg {
                MeabyMulti::Multi(multi) => {
                    get_sprite_trait_from_multi_bg(
                        multi,
                        last_group_index,
                        &image,
                        image_resource,
                        &tileset,
                    )
                }
                MeabyMulti::Single(bg) => {
                    get_sprite_trait_from_single_bg(
                        &bg,
                        last_group_index,
                        &image,
                        image_resource,
                        &tileset,
                    )
                }
            }
        }
        None => None
    };

    return (get_fg, get_bg);
}

fn get_multi_fg_and_bg(
    image_resource: &mut ResMut<Assets<Image>>,
    fg: &MeabyMulti<MeabyWeighted<i32>>,
    bg: &Option<MeabyMulti<MeabyWeighted<i32>>>,
    last_group_index: i32,
    image: &DynamicImage,
    tileset: &LegacyTileset,
) -> (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>) {
    let get_fg = match fg {
        MeabyMulti::Single(fg) => {
            // Seriously? Why would you not just put the same id in the list four times?
            let get_fg = get_sprite_trait_from_single_fg(
                &fg,
                last_group_index,
                &image,
                image_resource,
                &tileset,
            ).unwrap();

            vec![get_fg.clone(), get_fg.clone(), get_fg.clone(), get_fg.clone()]
        }
        MeabyMulti::Multi(v) => {
            // Direction is NW, SW, SE, NE
            let mut getters = Vec::new();

            for fg in v.iter() {
                let arc = get_sprite_trait_from_single_fg(
                    &fg,
                    last_group_index,
                    &image,
                    image_resource,
                    &tileset,
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
                        last_group_index,
                        &image,
                        image_resource,
                        &tileset,
                    )
                }
                MeabyMulti::Single(bg) => {
                    get_sprite_trait_from_single_bg(
                        bg,
                        last_group_index,
                        &image,
                        image_resource,
                        &tileset,
                    )
                }
            }
        }
        None => None
    };

    return (get_fg, get_bg);
}

impl TilesetLoader<LegacyTileset> for LegacyTilesetLoader {
    fn get_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, SpriteType>, anyhow::Error> {
        let tileset = self.load().unwrap();
        let mut textures: HashMap<TileId, SpriteType> = HashMap::new();

        let mut last_group_index = 13328;
        // TODO REPLACE
        for group in tileset.tiles.get(6) {
            let image = Reader::open(self.path.join(PathBuf::from_str(group.file.as_str()).unwrap()))
                .unwrap()
                .decode()
                .unwrap();

            let mut amount_of_tiles = 0;

            for tile in group.tiles.iter() {
                let get_main_fg: Arc<dyn GetForeground> = match &tile.fg.as_ref().unwrap() {
                    MeabyMulti::Single(fg) => {
                        match get_sprite_trait_from_single_fg(
                            &fg,
                            last_group_index,
                            &image,
                            image_resource,
                            &tileset,
                        ) {
                            None => {
                                warn!("Could not load sprite {:?} (out of range min fg value: {:?} actual fg value: {:?})", tile.id, last_group_index, fg);
                                continue;
                            }
                            Some(a) => a
                        }
                    }
                    MeabyMulti::Multi(fg) => {
                        match get_sprite_trait_from_multi_fg(
                            &fg,
                            last_group_index,
                            &image,
                            image_resource,
                            &tileset,
                        ) {
                            None => {
                                warn!("Could not load sprite {:?} (out of range min fg value: {:?} actual fg value: {:?})", tile.id, last_group_index, fg);
                                continue;
                            }
                            Some(a) => a
                        }
                    }
                };

                let get_main_bg: Option<Arc<dyn GetBackground>> = match &tile.bg {
                    None => None,
                    Some(bg) => {
                        match bg {
                            MeabyMulti::Single(bg) => {
                                match get_sprite_trait_from_single_bg(
                                    &bg,
                                    last_group_index,
                                    &image,
                                    image_resource,
                                    &tileset,
                                ) {
                                    None => {
                                        warn!("Could not load sprite {:?} (out of range min bg value: {:?} actual bg value: {:?})", tile.id, last_group_index, bg);
                                        None
                                    }
                                    Some(a) => Some(a)
                                }
                            }
                            MeabyMulti::Multi(bg) => {
                                match get_sprite_trait_from_multi_bg(
                                    &bg,
                                    last_group_index,
                                    &image,
                                    image_resource,
                                    &tileset,
                                ) {
                                    None => {
                                        warn!("Could not load sprite {:?} (out of range min bg value: {:?} actual bg value: {:?})", tile.id, last_group_index, bg);
                                        None
                                    }
                                    Some(a) => Some(a)
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
                                textures.insert(
                                    TileId { 0: v.clone() },
                                    SpriteType::Single(Sprite {
                                        fg: get_main_fg.clone(),
                                        bg: get_main_bg.clone(),
                                    }),
                                );
                                amount_of_tiles += 1;
                            }
                            MeabyMulti::Multi(v) => {
                                for value in v.iter() {
                                    debug!("Loaded tile {:?}", value);
                                    textures.insert(
                                        TileId { 0: value.clone() },
                                        SpriteType::Single(Sprite {
                                            fg: get_main_fg.clone(),
                                            bg: get_main_bg.clone(),
                                        }),
                                    );
                                    amount_of_tiles += 1;
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

                                match additional_tile.id.as_str() {
                                    "center" => {
                                        let (get_fg, get_bg) = get_single_fg_and_bg(
                                            image_resource,
                                            fg,
                                            bg,
                                            last_group_index,
                                            &image,
                                            &tileset,
                                        );
                                        center = Some(Sprite {
                                            fg: get_fg.unwrap(),
                                            bg: get_bg,
                                        })
                                    }
                                    "corner" => {
                                        let v = get_multi_fg_and_bg(
                                            image_resource,
                                            fg,
                                            bg,
                                            last_group_index,
                                            &image,
                                            &tileset,
                                        );
                                        corner = Some(Corner::from(v))
                                    }
                                    "t_connection" => {
                                        let v = get_multi_fg_and_bg(
                                            image_resource,
                                            fg,
                                            bg,
                                            last_group_index,
                                            &image,
                                            &tileset,
                                        );
                                        t_connection = Some(FullCardinal::from(v));
                                    }
                                    "edge" => {
                                        let v = get_multi_fg_and_bg(
                                            image_resource,
                                            fg,
                                            bg,
                                            last_group_index,
                                            &image,
                                            &tileset,
                                        );
                                        edge = Some(Edge::from(v));
                                    }
                                    "end_piece" => {
                                        let v = get_multi_fg_and_bg(
                                            image_resource,
                                            fg,
                                            bg,
                                            last_group_index,
                                            &image,
                                            &tileset,
                                        );
                                        end_piece = Some(FullCardinal::from(v));
                                    }
                                    "unconnected" => {
                                        let (get_fg, get_bg) = get_single_fg_and_bg(
                                            image_resource,
                                            fg,
                                            bg,
                                            last_group_index,
                                            &image,
                                            &tileset,
                                        );
                                        unconnected = Some(Sprite {
                                            fg: get_fg.unwrap(),
                                            bg: get_bg,
                                        });
                                    }
                                    _ => { panic!("Go Unexpected id {}", additional_tile.id) }
                                }
                            }

                            let default_sprite = Sprite {
                                fg: get_main_fg.clone(),
                                bg: get_main_bg.clone(),
                            };

                            textures.insert(
                                TileId { 0: id.clone() },
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
                            amount_of_tiles += 1;
                        }
                    }
                }
            }
        };

        return Ok(textures);
    }
}

