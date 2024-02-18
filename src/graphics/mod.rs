use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::{Assets, Image, ResMut, Resource};

use crate::common::{Coordinates, GetRandom, TileId};
use crate::common::io::Load;
use crate::graphics::tileset::legacy::{GetBackground, GetForeground, LegacyTileset, SingleForeground};
use crate::graphics::tileset::TilesetLoader;
use crate::project::resources::Project;
use crate::region_settings::RegionSettings;

pub(crate) mod tileset;

// Not sure if this is the best way to do this
#[derive(Clone)]
pub struct Sprite {
    pub fg: Option<Arc<dyn GetForeground>>,
    pub bg: Option<Arc<dyn GetBackground>>,
}

pub struct FullCardinal {
    pub north: Sprite,
    pub east: Sprite,
    pub south: Sprite,
    pub west: Sprite,
}

impl From<(Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>)> for FullCardinal {
    fn from(value: (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>)) -> Self {
        return FullCardinal {
            north: Sprite {
                fg: value.0.get(0).cloned(),
                bg: value.1.clone(),
            },
            west: Sprite {
                fg: value.0.get(1).cloned(),
                bg: value.1.clone(),
            },
            south: Sprite {
                fg: value.0.get(2).cloned(),
                bg: value.1.clone(),
            },
            east: Sprite {
                fg: value.0.get(3).cloned(),
                bg: value.1,
            },
        };
    }
}


pub struct Corner {
    pub north_west: Sprite,
    pub south_west: Sprite,
    pub south_east: Sprite,
    pub north_east: Sprite,
}

impl From<(Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>)> for Corner {
    fn from(value: (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>)) -> Self {
        return Corner {
            north_west: Sprite {
                fg: value.0.get(0).cloned(),
                bg: value.1.clone(),
            },
            south_west: Sprite {
                fg: value.0.get(1).cloned(),
                bg: value.1.clone(),
            },
            south_east: Sprite {
                fg: value.0.get(2).cloned(),
                bg: value.1.clone(),
            },
            north_east: Sprite {
                fg: value.0.get(3).cloned(),
                bg: value.1.clone(),
            },
        };
    }
}

pub struct Edge {
    pub north_south: Sprite,
    pub east_west: Sprite,
}

impl From<(Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>)> for Edge {
    fn from(value: (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>)) -> Self {
        return Self {
            north_south: Sprite { fg: value.0.get(0).cloned(), bg: value.1.clone() },
            east_west: Sprite { fg: value.0.get(1).cloned(), bg: value.1.clone() },
        };
    }
}

pub enum SpriteType {
    Single(Sprite),
    Multitile {
        center: Sprite,
        corner: Corner,
        t_connection: FullCardinal,
        edge: Edge,
        end_piece: FullCardinal,
        unconnected: Sprite,
    },
}

pub enum TileSprite<'a> {
    Exists {
        terrain: Option<&'a Sprite>,
        furniture: Option<&'a Sprite>,
        items: Option<&'a Sprite>,
        toilets: Option<&'a Sprite>,
    },
    Default(&'a Sprite),
    // Reserved for " " chars
    Empty,
}

pub trait GetTexture: Send + Sync {
    fn get_textures(&self, project: &Project, character: &char, coordinates: &Coordinates) -> TileSprite {
        if character == &' ' { return TileSprite::Empty; }

        let terrain = self.get_terrain(project, character, coordinates);
        let furniture = self.get_furniture(project, character, coordinates);
        let items = self.get_item(project, character, coordinates);
        let toilets = self.get_toilets(project, character, coordinates);

        if terrain.is_none() && furniture.is_none() && items.is_none() && toilets.is_none() {
            // Return Default Texture
            return TileSprite::Default(self.get_fallback_texture(character));
        }

        return TileSprite::Exists {
            terrain,
            furniture,
            items,
            toilets,
        };
    }

    fn get_terrain(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite>;
    fn get_furniture(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite>;
    fn get_item(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite>;
    fn get_toilets(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite>;
    fn get_fallback_texture(&self, character: &char) -> &Sprite;
}

pub struct LegacyTextures {
    textures: HashMap<TileId, SpriteType>,
    fallback_textures: HashMap<String, Sprite>,
    region_settings: RegionSettings,
}

impl LegacyTextures {
    pub fn new(loader: impl TilesetLoader<LegacyTileset, i32>, region_settings: impl Load<RegionSettings>, image_resource: &mut ResMut<Assets<Image>>) -> Self {
        let textures = loader.assign_textures(image_resource).unwrap();
        let fallback_textures = loader.load_fallback_textures(image_resource).unwrap();

        let mut fallback_sprites: HashMap<String, Sprite> = HashMap::new();

        for (key, image) in fallback_textures {
            fallback_sprites.insert(
                key,
                Sprite {
                    fg: Some(Arc::new(SingleForeground::new(image))),
                    bg: None,
                },
            );
        };

        return Self {
            textures,
            fallback_textures: fallback_sprites,
            region_settings: region_settings.load().unwrap(),
        };
    }
}

fn get_sprite_from_sprite_type<'a>(
    project: &Project,
    coordinates: &Coordinates,
    character: &char,
    sprite_type: &'a SpriteType,
) -> &'a Sprite {
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

impl GetTexture for LegacyTextures {
    fn get_terrain(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite> {
        match &project.map_entity.get_ids(character).terrain {
            None => return None,
            Some(terrain) => {
                if let Some(terrain_region) = self.region_settings.region_terrain_and_furniture.terrain.get(&terrain.0) {
                    let sprite_type = match self.textures.get(terrain_region.get_random_weighted().unwrap()) {
                        None => return None,
                        Some(s) => s
                    };

                    return Some(get_sprite_from_sprite_type(
                        project,
                        coordinates,
                        character,
                        sprite_type,
                    ));
                }

                let sprite_type = match self.textures.get(terrain) {
                    None => return None,
                    Some(s) => s
                };

                return Some(get_sprite_from_sprite_type(
                    project,
                    coordinates,
                    character,
                    sprite_type,
                ));
            }
        };
    }

    fn get_furniture(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite> {
        match &project.map_entity.get_ids(character).furniture {
            None => return None,
            Some(furniture) => {
                let sprite_type = match self.textures.get(furniture) {
                    None => return None,
                    Some(s) => s
                };

                return Some(get_sprite_from_sprite_type(
                    project,
                    coordinates,
                    character,
                    sprite_type,
                ));
            }
        };
    }

    fn get_item(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite> {
        match &project.map_entity.get_ids(character).item {
            None => return None,
            Some(item) => {
                let sprite_type = match self.textures.get(item) {
                    None => return None,
                    Some(s) => s
                };

                return Some(get_sprite_from_sprite_type(
                    project,
                    coordinates,
                    character,
                    sprite_type,
                ));
            }
        };
    }

    fn get_toilets(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite> {
        match &project.map_entity.get_ids(character).toilet {
            None => return None,
            Some(toilet) => {
                let sprite_type = match self.textures.get(toilet) {
                    None => return None,
                    Some(s) => s
                };

                return Some(get_sprite_from_sprite_type(
                    project,
                    coordinates,
                    character,
                    sprite_type,
                ));
            }
        };
    }

    fn get_fallback_texture(&self, character: &char) -> &Sprite {
        return self.fallback_textures.get(&format!("{}_WHITE", &character.to_string().to_uppercase())).unwrap_or(
            self.fallback_textures.get("?_WHITE").unwrap()
        );
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