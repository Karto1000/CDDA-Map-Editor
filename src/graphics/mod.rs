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
    pub offset_x: i32,
    pub offset_y: i32,
    pub is_animated: bool,
}

pub struct FullCardinal {
    pub north: Sprite,
    pub east: Sprite,
    pub south: Sprite,
    pub west: Sprite,
}

impl From<(Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>, i32, i32, bool)> for FullCardinal {
    fn from(value: (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>, i32, i32, bool)) -> Self {
        return FullCardinal {
            north: Sprite {
                fg: value.0.get(0).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
            west: Sprite {
                fg: value.0.get(1).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
            south: Sprite {
                fg: value.0.get(2).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,

            },
            east: Sprite {
                fg: value.0.get(3).cloned(),
                bg: value.1,
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
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

impl From<(Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>, i32, i32, bool)> for Corner {
    fn from(value: (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>, i32, i32, bool)) -> Self {
        return Corner {
            north_west: Sprite {
                fg: value.0.get(0).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
            south_west: Sprite {
                fg: value.0.get(1).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
            south_east: Sprite {
                fg: value.0.get(2).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
            north_east: Sprite {
                fg: value.0.get(3).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
        };
    }
}

pub struct Edge {
    pub north_south: Sprite,
    pub east_west: Sprite,
}

impl From<(Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>, i32, i32, bool)> for Edge {
    fn from(value: (Vec<Arc<dyn GetForeground>>, Option<Arc<dyn GetBackground>>, i32, i32, bool)) -> Self {
        return Self {
            north_south: Sprite {
                fg: value.0.get(0).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
            east_west: Sprite {
                fg: value.0.get(1).cloned(),
                bg: value.1.clone(),
                offset_x: value.2,
                offset_y: value.3,
                is_animated: value.4,
            },
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
    Fallback(&'a Sprite),
    // Reserved for " " chars with no fill_ter
    Empty,
}

pub trait GetTexture: Send + Sync {
    fn get_textures(&self, project: &Project, character: &char, coordinates: &Coordinates) -> TileSprite {
        let mut terrain = self.get_terrain(project, character, coordinates);
        let furniture = self.get_furniture(project, character, coordinates);
        let items = self.get_item(project, character, coordinates);
        let toilets = self.get_toilets(project, character, coordinates);

        match &project.map_entity.fill {
            None => {}
            Some(v) => {
                // Add fill_ter terrain texture when terrain does not exist
                if terrain.is_none() {
                    terrain = Some(self.get_texture_from_tile_id(project, coordinates, v).unwrap())
                }
            }
        }

        if terrain.is_none() && furniture.is_none() && items.is_none() && toilets.is_none() {
            if project.map_entity.fill.is_none() {
                return TileSprite::Empty;
            }

            // Return Default Texture
            return TileSprite::Fallback(self.get_fallback_texture(character));
        }

        return TileSprite::Exists {
            terrain,
            furniture,
            items,
            toilets,
        };
    }

    fn get_texture_from_tile_id(&self, project: &Project, coordinates: &Coordinates, id: &TileId) -> Option<&Sprite>;
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
                    offset_x: 0,
                    offset_y: 0,
                    is_animated: false
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

macro_rules! define_sprite_from_sprite_type {
    ($field: ident, $ident: ident) => {
        fn $ident<'a>(
            project: &Project,
            coordinates: &Coordinates,
            character: &char,
            sprite_type: &'a SpriteType) -> &'a Sprite {
            return match sprite_type {
                SpriteType::Single(s) => s,
                SpriteType::Multitile { center, corner, t_connection, edge, end_piece, unconnected } => {
                    let tiles_around = project.map_entity.get_tiles_around(coordinates);

                    // TODO handle errors
                    let fill = project.map_entity.fill.as_ref().unwrap();

                    macro_rules! match_tiles_around {
                        ($name: ident, $num: expr) => {
                           let $name = match tiles_around.get($num).unwrap().0 {
                               None => false,
                               Some(t)  => {
                                   let ids = project.map_entity.get_ids(&t.character);
                            
                                    let is_same_character = t.character == *character;
                                    let is_same_id = match &ids.$field {
                                        None => false,
                                        Some(t) => t == fill
                                    };
                                    
                                    is_same_character || is_same_id || (ids.$field.is_none() && project.map_entity.get_ids(character).$field.is_none()) 
                               }
                           };
                        }
                    }

                    match_tiles_around!(is_tile_ontop_same_type, 0);
                    match_tiles_around!(is_tile_right_same_type, 1);
                    match_tiles_around!(is_tile_below_same_type, 2);
                    match_tiles_around!(is_tile_left_same_type, 3);


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
    };
}

define_sprite_from_sprite_type!(terrain, get_terrain_sprite_from_sprite_type);
define_sprite_from_sprite_type!(furniture, get_furniture_sprite_from_sprite_type);
define_sprite_from_sprite_type!(item, get_item_sprite_from_sprite_type);
define_sprite_from_sprite_type!(toilet, get_toilet_sprite_from_sprite_type);


impl GetTexture for LegacyTextures {
    fn get_texture_from_tile_id(&self, project: &Project, coordinates: &Coordinates, id: &TileId) -> Option<&Sprite> {
        let sprite_type = match self.textures.get(id) {
            None => return None,
            Some(s) => s
        };

        return Some(get_terrain_sprite_from_sprite_type(
            project,
            coordinates,
            &' ',
            sprite_type,
        ));
    }

    fn get_terrain(&self, project: &Project, character: &char, coordinates: &Coordinates) -> Option<&Sprite> {
        match &project.map_entity.get_ids(character).terrain {
            None => return None,
            Some(terrain) => {
                if let Some(terrain) = self.region_settings.get_random_terrain_from_region(&terrain.0) {
                    let sprite_type = match self.textures.get(terrain) {
                        None => return None,
                        Some(s) => s
                    };

                    return Some(get_terrain_sprite_from_sprite_type(
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

                return Some(get_terrain_sprite_from_sprite_type(
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
                if let Some(furniture) = self.region_settings.get_random_furniture_from_region(&furniture.0) {
                    let sprite_type = match self.textures.get(furniture) {
                        None => return None,
                        Some(s) => s
                    };

                    return Some(get_furniture_sprite_from_sprite_type(
                        project,
                        coordinates,
                        character,
                        sprite_type,
                    ));
                }

                let sprite_type = match self.textures.get(furniture) {
                    None => return None,
                    Some(s) => s
                };

                return Some(get_furniture_sprite_from_sprite_type(
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

                return Some(get_item_sprite_from_sprite_type(
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

                return Some(get_toilet_sprite_from_sprite_type(
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