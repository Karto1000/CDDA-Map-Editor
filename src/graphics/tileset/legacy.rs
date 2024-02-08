use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::FromStr;

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Image, ResMut, Vec2};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{DynamicImage, GenericImageView};
use image::io::Reader;
use serde::Deserialize;
use serde_json::Value;

use crate::common::{MeabyWeighted, TileId};
use crate::graphics::{Corner, Edge, FullCardinal, SpriteType};
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

fn get_xy_from_fg(fg: &i32, last_group_index: i32) -> Vec2 {
    let local_tile_index: u32 = (fg - last_group_index) as u32;

    return Vec2::new(
        (local_tile_index % AMOUNT_OF_SPRITES_PER_ROW) as f32,
        ((local_tile_index / AMOUNT_OF_SPRITES_PER_ROW) as f32).floor(),
    );
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

impl TilesetLoader<LegacyTileset> for LegacyTilesetLoader {
    fn get_textures(&self, image_resource: &mut ResMut<Assets<Image>>) -> Result<HashMap<TileId, SpriteType>, anyhow::Error> {
        let tileset = self.load().unwrap();
        let mut textures: HashMap<TileId, SpriteType> = HashMap::new();

        let mut last_group_index = 13327;
        // TODO REPLACE
        for group in tileset.tiles.get(6) {
            let image = Reader::open(self.path.join(PathBuf::from_str(group.file.as_str()).unwrap()))
                .unwrap()
                .decode()
                .unwrap();

            let mut amount_of_tiles = 0;

            for tile in group.tiles.iter() {
                let fg = match &tile.fg.as_ref().unwrap() {
                    MeabyMulti::Single(fg) => fg,
                    // TODO, Just here for testing, implement this later
                    MeabyMulti::Multi(multi) => multi.first().unwrap()
                };

                let xy = match fg {
                    MeabyWeighted::NotWeighted(fg) => {
                        // TODO: Figure out what to do if the tile is inside another file
                        if fg < &last_group_index {
                            continue;
                        }
                        get_xy_from_fg(fg, last_group_index)
                    }
                    // TODO Figure out how weights work here
                    MeabyWeighted::Weighted(weight) => {
                        // TODO: Figure out what to do if the tile is inside another file
                        if weight.value < last_group_index {
                            continue;
                        }
                        get_xy_from_fg(&weight.value, last_group_index)
                    }
                };

                let main_image = get_image_from_tileset(
                    &image,
                    xy.x as u32 * tileset.info.tile_width,
                    xy.y as u32 * tileset.info.tile_height,
                    tileset.info.tile_width,
                    tileset.info.tile_height,
                );

                match &tile.additional_tiles {
                    None => {
                        match &tile.id {
                            MeabyMulti::Single(v) => {
                                textures.insert(TileId { 0: v.clone() }, SpriteType::Single(image_resource.add(main_image)));
                                amount_of_tiles += 1;
                            }
                            MeabyMulti::Multi(v) => {
                                // TODO Actually Implement this
                                for value in v.iter() {
                                    textures.insert(TileId { 0: value.clone() }, SpriteType::Single(image_resource.add(main_image.clone())));
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
                            let mut center: Option<Handle<Image>> = None;
                            let mut corner: Option<Corner> = None;
                            let mut t_connection: Option<FullCardinal> = None;
                            let mut edge: Option<Edge> = None;
                            let mut end_piece: Option<FullCardinal> = None;
                            let mut unconnected: Option<Handle<Image>> = None;

                            for additional_tile in additional_tiles.iter() {
                                match additional_tile.fg.as_ref() {
                                    Some(fg) => {
                                        match additional_tile.id.as_str() {
                                            "center" => {
                                                let fg = match fg {
                                                    // TODO: Figure out how weights work here
                                                    MeabyMulti::Multi(multi) => multi.first().unwrap(),
                                                    MeabyMulti::Single(fg) => fg
                                                };

                                                let xy = match fg {
                                                    MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                    MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                };

                                                let image = get_image_from_tileset(
                                                    &image,
                                                    xy.x as u32 * tileset.info.tile_width,
                                                    xy.y as u32 * tileset.info.tile_height,
                                                    tileset.info.tile_width,
                                                    tileset.info.tile_height,
                                                );

                                                center = Some(image_resource.add(image));
                                            }
                                            "corner" => {
                                                match fg {
                                                    MeabyMulti::Single(fg) => {
                                                        // Seriously? Why would you not just put the same id in the list four times?

                                                        let xy = match fg {
                                                            MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                            MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                        };

                                                        let image = get_image_from_tileset(
                                                            &image,
                                                            xy.x as u32 * tileset.info.tile_width,
                                                            xy.y as u32 * tileset.info.tile_height,
                                                            tileset.info.tile_width,
                                                            tileset.info.tile_height,
                                                        );

                                                        corner = Some(Corner {
                                                            north_west: image_resource.add(image.clone()),
                                                            south_west: image_resource.add(image.clone()),
                                                            south_east: image_resource.add(image.clone()),
                                                            north_east: image_resource.add(image.clone()),
                                                        })
                                                    }
                                                    MeabyMulti::Multi(v) => {
                                                        // Direction is NW, SW, SE, NE
                                                        let mut images = Vec::new();

                                                        for fg in v.iter() {
                                                            let xy = match fg {
                                                                MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                                MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                            };

                                                            let image = get_image_from_tileset(
                                                                &image,
                                                                xy.x as u32 * tileset.info.tile_width,
                                                                xy.y as u32 * tileset.info.tile_height,
                                                                tileset.info.tile_width,
                                                                tileset.info.tile_height,
                                                            );

                                                            images.push(image);
                                                        }

                                                        // I assume that the ordering of the fg values is always the same
                                                        corner = Some(Corner {
                                                            north_west: image_resource.add(images.get(0).unwrap().clone()),
                                                            south_west: image_resource.add(images.get(1).unwrap().clone()),
                                                            south_east: image_resource.add(images.get(2).unwrap().clone()),
                                                            north_east: image_resource.add(images.get(3).unwrap().clone()),
                                                        })
                                                    }
                                                }
                                            }
                                            "t_connection" => {
                                                match fg {
                                                    MeabyMulti::Single(fg) => {
                                                        let xy = match fg {
                                                            MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                            MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                        };

                                                        let image = get_image_from_tileset(
                                                            &image,
                                                            xy.x as u32 * tileset.info.tile_width,
                                                            xy.y as u32 * tileset.info.tile_height,
                                                            tileset.info.tile_width,
                                                            tileset.info.tile_height,
                                                        );

                                                        t_connection = Some(FullCardinal {
                                                            north: image_resource.add(image.clone()),
                                                            west: image_resource.add(image.clone()),
                                                            south: image_resource.add(image.clone()),
                                                            east: image_resource.add(image.clone()),
                                                        })
                                                    }
                                                    MeabyMulti::Multi(v) => {
                                                        // Direction is N, W, S, E
                                                        let mut images = Vec::new();

                                                        for fg in v.iter() {
                                                            let xy = match fg {
                                                                MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                                MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                            };

                                                            let image = get_image_from_tileset(
                                                                &image,
                                                                xy.x as u32 * tileset.info.tile_width,
                                                                xy.y as u32 * tileset.info.tile_height,
                                                                tileset.info.tile_width,
                                                                tileset.info.tile_height,
                                                            );

                                                            images.push(image);
                                                        }

                                                        // I assume that the ordering of the fg values is always the same
                                                        t_connection = Some(FullCardinal {
                                                            north: image_resource.add(images.get(0).unwrap().clone()),
                                                            west: image_resource.add(images.get(1).unwrap().clone()),
                                                            south: image_resource.add(images.get(2).unwrap().clone()),
                                                            east: image_resource.add(images.get(3).unwrap().clone()),
                                                        })
                                                    }
                                                }
                                            }
                                            "edge" => {
                                                match fg {
                                                    MeabyMulti::Single(fg) => {
                                                        let xy = match fg {
                                                            MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                            MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                        };

                                                        let image = get_image_from_tileset(
                                                            &image,
                                                            xy.x as u32 * tileset.info.tile_width,
                                                            xy.y as u32 * tileset.info.tile_height,
                                                            tileset.info.tile_width,
                                                            tileset.info.tile_height,
                                                        );

                                                        edge = Some(Edge {
                                                            north_south: image_resource.add(image.clone()),
                                                            east_west: image_resource.add(image.clone()),
                                                        })
                                                    }
                                                    MeabyMulti::Multi(v) => {
                                                        // Direction is NS, EW
                                                        let mut images = Vec::new();

                                                        for fg in v.iter() {
                                                            let xy = match fg {
                                                                MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                                MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                            };

                                                            let image = get_image_from_tileset(
                                                                &image,
                                                                xy.x as u32 * tileset.info.tile_width,
                                                                xy.y as u32 * tileset.info.tile_height,
                                                                tileset.info.tile_width,
                                                                tileset.info.tile_height,
                                                            );

                                                            images.push(image);
                                                        }

                                                        // I assume that the ordering of the fg values is always the same
                                                        edge = Some(Edge {
                                                            north_south: image_resource.add(images.get(0).unwrap().clone()),
                                                            east_west: image_resource.add(images.get(1).unwrap().clone()),
                                                        })
                                                    }
                                                }
                                            }
                                            "end_piece" => {
                                                match fg {
                                                    MeabyMulti::Single(fg) => {
                                                        let xy = match fg {
                                                            MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                            MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                        };

                                                        let image = get_image_from_tileset(
                                                            &image,
                                                            xy.x as u32 * tileset.info.tile_width,
                                                            xy.y as u32 * tileset.info.tile_height,
                                                            tileset.info.tile_width,
                                                            tileset.info.tile_height,
                                                        );

                                                        end_piece = Some(FullCardinal {
                                                            north: image_resource.add(image.clone()),
                                                            west: image_resource.add(image.clone()),
                                                            south: image_resource.add(image.clone()),
                                                            east: image_resource.add(image.clone()),
                                                        })
                                                    }
                                                    MeabyMulti::Multi(v) => {
                                                        // Direction is N, W, S, E
                                                        let mut images = Vec::new();

                                                        for fg in v.iter() {
                                                            let xy = match fg {
                                                                MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                                MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                            };

                                                            let image = get_image_from_tileset(
                                                                &image,
                                                                xy.x as u32 * tileset.info.tile_width,
                                                                xy.y as u32 * tileset.info.tile_height,
                                                                tileset.info.tile_width,
                                                                tileset.info.tile_height,
                                                            );

                                                            images.push(image);
                                                        }

                                                        // I assume that the ordering of the fg values is always the same
                                                        end_piece = Some(FullCardinal {
                                                            north: image_resource.add(images.get(0).unwrap().clone()),
                                                            west: image_resource.add(images.get(1).unwrap().clone()),
                                                            south: image_resource.add(images.get(2).unwrap().clone()),
                                                            east: image_resource.add(images.get(3).unwrap().clone()),
                                                        })
                                                    }
                                                }
                                            }
                                            "unconnected" => {
                                                let fg = match fg {
                                                    // TODO: Figure out how weights work here
                                                    MeabyMulti::Multi(multi) => multi.first().unwrap(),
                                                    MeabyMulti::Single(fg) => fg
                                                };

                                                let xy = match fg {
                                                    MeabyWeighted::NotWeighted(fg) => get_xy_from_fg(fg, last_group_index),
                                                    MeabyWeighted::Weighted(weight) => get_xy_from_fg(&weight.value, last_group_index)
                                                };

                                                let image = get_image_from_tileset(
                                                    &image,
                                                    xy.x as u32 * tileset.info.tile_width,
                                                    xy.y as u32 * tileset.info.tile_height,
                                                    tileset.info.tile_width,
                                                    tileset.info.tile_height,
                                                );

                                                unconnected = Some(image_resource.add(image));
                                            }

                                            _ => { panic!("Go Unexpected id {}", additional_tile.id) }
                                        }
                                    }
                                    None => {}
                                }
                            }

                            textures.insert(
                                TileId { 0: id.clone() },
                                SpriteType::Multitile {
                                    center: center.unwrap_or(image_resource.add(main_image.clone())),
                                    corner: corner.unwrap_or(Corner {
                                        north_west: image_resource.add(main_image.clone()),
                                        south_west: image_resource.add(main_image.clone()),
                                        south_east: image_resource.add(main_image.clone()),
                                        north_east: image_resource.add(main_image.clone()),
                                    }),
                                    t_connection: t_connection.unwrap_or(FullCardinal {
                                        north: image_resource.add(main_image.clone()),
                                        east: image_resource.add(main_image.clone()),
                                        south: image_resource.add(main_image.clone()),
                                        west: image_resource.add(main_image.clone()),
                                    }),
                                    edge: edge.unwrap_or(Edge {
                                        north_south: image_resource.add(main_image.clone()),
                                        east_west: image_resource.add(main_image.clone()),
                                    }),
                                    end_piece: end_piece.unwrap_or(FullCardinal {
                                        north: image_resource.add(main_image.clone()),
                                        east: image_resource.add(main_image.clone()),
                                        south: image_resource.add(main_image.clone()),
                                        west: image_resource.add(main_image.clone()),
                                    }),
                                    unconnected: unconnected.unwrap_or(image_resource.add(main_image.clone())),
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

