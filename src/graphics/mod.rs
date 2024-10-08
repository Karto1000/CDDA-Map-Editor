use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::{Assets, Image, ResMut, Resource};

use crate::common::{Coordinates, TileId};
use crate::common::io::Load;
use crate::program::data::CDDAData;
use crate::graphics::tileset::{GetBackground, GetForeground, TilesetLoader};
use crate::graphics::tileset::legacy::{LegacyTileset, SingleForeground};
use crate::project::data::Project;
use crate::region_settings::data::RegionSettings;

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

pub enum SpriteState<'a> {
    /// Sprite is explicitly defined in either map object or palette
    Defined(&'a Sprite),
    /// Sprites that are defined, but not found in textures
    TextureNotFound,
    /// When no mapping has been defined
    NotMapped,
}

pub trait GetTexture: Send + Sync {
    fn get_textures(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> TileSprite {
        let terrain = self.get_terrain(project, cdda_data, character, coordinates);
        let furniture = self.get_furniture(project, cdda_data, character, coordinates);
        let items = self.get_item(project, cdda_data, character, coordinates);
        let toilets = self.get_toilets(project, cdda_data, character, coordinates);

        let terrain_sprite: Option<&Sprite> = match terrain {
            SpriteState::Defined(s) => Some(s),
            SpriteState::TextureNotFound => Some(self.get_fallback_texture(character)),
            SpriteState::NotMapped => {
                match &project.map_entity.object().fill_ter {
                    None => None,
                    Some(fill) => self.get_terrain_texture_from_tile_id(project, cdda_data, coordinates, fill)
                }
            }
        };

        let furniture_sprite: Option<&Sprite> = match furniture {
            SpriteState::Defined(s) => Some(s),
            SpriteState::TextureNotFound => Some(self.get_fallback_texture(character)),
            SpriteState::NotMapped => None
        };

        let items_sprite: Option<&Sprite> = match items {
            SpriteState::Defined(s) => Some(s),
            SpriteState::TextureNotFound => Some(self.get_fallback_texture(character)),
            SpriteState::NotMapped => None
        };

        let toilets_sprite: Option<&Sprite> = match toilets {
            SpriteState::Defined(s) => Some(s),
            SpriteState::TextureNotFound => Some(self.get_fallback_texture(character)),
            SpriteState::NotMapped => None
        };

        if terrain_sprite.is_none() && furniture_sprite.is_none() && items_sprite.is_none() && toilets_sprite.is_none() {
            return TileSprite::Empty;
        }

        return TileSprite::Exists {
            terrain: terrain_sprite,
            furniture: furniture_sprite,
            items: items_sprite,
            toilets: toilets_sprite,
        };
    }

    fn get_terrain_representation(&self, tile_id: &TileId) -> &Sprite;

    fn get_terrain_texture_from_tile_id(&self, project: &Project, cdda_data: &CDDAData, coordinates: &Coordinates, id: &TileId) -> Option<&Sprite>;
    fn get_terrain(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState;
    fn get_furniture(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState;
    fn get_item(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState;
    fn get_toilets(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState;
    fn get_fallback_texture(&self, character: &char) -> &Sprite;
}

pub struct LegacyTextures {
    textures: HashMap<TileId, SpriteType>,
    fallback_textures: HashMap<String, Sprite>,
    region_settings: RegionSettings,
}

impl LegacyTextures {
    pub fn new(loader: impl TilesetLoader<LegacyTileset, i32>, region_settings: impl Load<RegionSettings>, image_resource: &mut ResMut<Assets<Image>>) -> Self {
        let textures = loader.load_sprite_handles(image_resource).unwrap();
        let fallback_textures = loader.load_fallback_textures().unwrap();

        let mut fallback_sprites: HashMap<String, Sprite> = HashMap::new();

        for (key, image) in fallback_textures {
            fallback_sprites.insert(
                key,
                Sprite {
                    fg: Some(Arc::new(SingleForeground::new(image_resource.add(image)))),
                    bg: None,
                    offset_x: 0,
                    offset_y: 0,
                    is_animated: false,
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

// TODO: Remove this macro and replace it with the define_get_sprite_from_sprite_type_and_tile_id macro
macro_rules! define_get_sprite_from_sprite_type {
    ($field: ident, $ident: ident, $is_terrain: literal) => {
        fn $ident<'a>(
            project: &Project,
            cdda_data: &CDDAData,
            coordinates: &Coordinates,
            character: &char,
            sprite_type: &'a SpriteType,
        ) -> &'a Sprite {
            return match sprite_type {
                SpriteType::Single(s) => s,
                SpriteType::Multitile { center, corner, t_connection, edge, end_piece, unconnected } => {
                    let tiles_around = project.map_entity.get_tiles_around(coordinates);
                    let field_this = project.map_entity.get_ids(cdda_data, character).$field;

                    macro_rules! match_tiles_around {
                        ($name: ident, $num: expr) => {
                           let $name = match tiles_around.get($num).unwrap().0 {
                               None => false,
                               Some(t)  => {
                                    let field_around = project.map_entity.get_ids(cdda_data, &t.character).$field;
                                    let is_same_character = t.character == *character;

                                    let is_this_filled = match (&field_this, &project.map_entity.object().fill_ter) {
                                        (None, Some(_)) => $is_terrain,
                                        (_, _) => false
                                    };

                                    let is_around_filled = match(&field_around, &project.map_entity.object().fill_ter) {
                                        (None, Some(_)) => $is_terrain,
                                        (_, _) => false
                                    };

                                    let is_same_id = match (&field_around, &field_this) {
                                        (Some(around), Some(this)) => *around == *this,
                                        (None, Some(this)) => is_around_filled && this == project.map_entity.object().fill_ter.as_ref().unwrap(),
                                        (Some(around), None) => is_this_filled && around == project.map_entity.object().fill_ter.as_ref().unwrap(),
                                        (None, None) => is_this_filled && is_around_filled
                                    };

                                    is_same_character || is_same_id
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

define_get_sprite_from_sprite_type!(terrain, get_terrain_sprite_from_sprite_type, true);
define_get_sprite_from_sprite_type!(furniture, get_furniture_sprite_from_sprite_type, false);
define_get_sprite_from_sprite_type!(item, get_item_sprite_from_sprite_type, false);
define_get_sprite_from_sprite_type!(toilet, get_toilet_sprite_from_sprite_type, false);

// TODO: Rename this
macro_rules! define_get_sprite_from_sprite_type_and_tile_id {
    ($field: ident, $ident: ident, $is_terrain: literal) => {
        fn $ident<'a>(
            project: &Project,
            cdda_data: &CDDAData,
            coordinates: &Coordinates,
            tile_id: Option<&TileId>,
            sprite_type: &'a SpriteType,
        ) -> &'a Sprite {
            return match sprite_type {
                SpriteType::Single(s) => s,
                SpriteType::Multitile { center, corner, t_connection, edge, end_piece, unconnected } => {
                    let tiles_around = project.map_entity.get_tiles_around(coordinates);

                    macro_rules! match_tiles_around {
                        ($name: ident, $num: expr) => {
                           let $name = match tiles_around.get($num).unwrap().0 {
                               None => false,
                               Some(t)  => {
                                    let field_around = project.map_entity.get_ids(cdda_data, &t.character).$field;

                                    let is_this_filled = match (&tile_id, &project.map_entity.object().fill_ter) {
                                        (None, Some(_)) => $is_terrain,
                                        (_, _) => false
                                    };

                                    let is_around_filled = match(&field_around, &project.map_entity.object().fill_ter) {
                                        (None, Some(_)) => $is_terrain,
                                        (_, _) => false
                                    };

                                    let is_same_id = match (&field_around, tile_id) {
                                        (Some(around), Some(this)) => *around == *this,
                                        (None, Some(this)) => is_around_filled && this == project.map_entity.object().fill_ter.as_ref().unwrap(),
                                        (Some(around), None) => is_this_filled && around == project.map_entity.object().fill_ter.as_ref().unwrap(),
                                        (None, None) => is_this_filled && is_around_filled
                                    };

                                    is_same_id
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

define_get_sprite_from_sprite_type_and_tile_id!(terrain, get_terrain_sprite_from_sprite_type_and_tile_id, true);

impl GetTexture for LegacyTextures {
    fn get_terrain_texture_from_tile_id(&self, project: &Project, cdda_data: &CDDAData, coordinates: &Coordinates, id: &TileId) -> Option<&Sprite> {
        let sprite_type = match self.textures.get(id) {
            None => return None,
            Some(s) => s
        };

        // TODO: Refactor this mess
        return Some(get_terrain_sprite_from_sprite_type_and_tile_id(
            project,
            cdda_data,
            coordinates,
            Some(id),
            sprite_type,
        ));
    }

    fn get_terrain_representation(&self, tile_or_region_id: &TileId) -> &Sprite {
        let tile_id = match self.region_settings.region_terrain_and_furniture.terrain.get(tile_or_region_id) {
            None => tile_or_region_id,
            Some(v) => v.iter().next().unwrap().0
        };
        
        let sprite_type = match self.textures.get(tile_id) {
            None => {
                let char: char = tile_or_region_id.splitn(2, "_")
                    .collect::<Vec<&str>>()
                    .get(1).map(|s| {s.chars().next().unwrap_or('?')})
                    .unwrap_or('?');
                return self.get_fallback_texture(&char);
            },
            Some(v) => v
        };

        return match sprite_type {
            SpriteType::Single(s) => s,
            SpriteType::Multitile { center, .. } => center
        }
    }

    fn get_terrain(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState {
        return match &project.map_entity.get_ids(cdda_data, character).terrain {
            None => SpriteState::NotMapped,
            Some(terrain) => {
                if let Some(terrain) = self.region_settings.get_random_terrain_from_region(&terrain) {
                    let sprite_type = match self.textures.get(terrain) {
                        None => return SpriteState::TextureNotFound,
                        Some(s) => s
                    };

                    return SpriteState::Defined(get_terrain_sprite_from_sprite_type(
                        project,
                        cdda_data,
                        coordinates,
                        character,
                        sprite_type,
                    ));
                }

                let sprite_type = match self.textures.get(terrain) {
                    None => return SpriteState::TextureNotFound,
                    Some(s) => s
                };

                SpriteState::Defined(get_terrain_sprite_from_sprite_type(
                    project,
                    cdda_data,
                    coordinates,
                    character,
                    sprite_type,
                ))
            }
        };
    }

    fn get_furniture(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState {
        return match &project.map_entity.get_ids(cdda_data, character).furniture {
            None => SpriteState::NotMapped,
            Some(furniture) => {
                if let Some(furniture) = self.region_settings.get_random_furniture_from_region(&furniture) {
                    let sprite_type = match self.textures.get(furniture) {
                        None => return SpriteState::TextureNotFound,
                        Some(s) => s
                    };

                    return SpriteState::Defined(get_furniture_sprite_from_sprite_type(
                        project,
                        cdda_data,
                        coordinates,
                        character,
                        sprite_type,
                    ));
                }

                let sprite_type = match self.textures.get(furniture) {
                    None => return SpriteState::TextureNotFound,
                    Some(s) => s
                };

                SpriteState::Defined(get_furniture_sprite_from_sprite_type(
                    project,
                    cdda_data,
                    coordinates,
                    character,
                    sprite_type,
                ))
            }
        };
    }

    fn get_item(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState {
        return match &project.map_entity.get_ids(cdda_data, character).item {
            None => SpriteState::NotMapped,
            Some(item) => {
                let sprite_type = match self.textures.get(item) {
                    None => return SpriteState::TextureNotFound,
                    Some(s) => s
                };

                SpriteState::Defined(get_item_sprite_from_sprite_type(
                    project,
                    cdda_data,
                    coordinates,
                    character,
                    sprite_type,
                ))
            }
        };
    }

    fn get_toilets(&self, project: &Project, cdda_data: &CDDAData, character: &char, coordinates: &Coordinates) -> SpriteState {
        return match &project.map_entity.get_ids(cdda_data, character).toilet {
            None => SpriteState::NotMapped,
            Some(toilet) => {
                let sprite_type = match self.textures.get(toilet) {
                    None => return SpriteState::TextureNotFound,
                    Some(s) => s
                };

                SpriteState::Defined(get_toilet_sprite_from_sprite_type(
                    project,
                    cdda_data,
                    coordinates,
                    character,
                    sprite_type,
                ))
            }
        };
    }

    fn get_fallback_texture(&self, character: &char) -> &Sprite {
        return self.fallback_textures.get(&format!("{}_WHITE", &character.to_string().to_uppercase())).unwrap_or(
            self.fallback_textures.get("?_WHITE").unwrap()
        );
    }
}

#[derive(Resource, Default)]
pub struct GraphicsResource {
    pub textures: Option<Box<dyn GetTexture>>,
}

impl GraphicsResource {
    pub fn new(tileset: Box<dyn GetTexture>) -> Self {
        return Self {
            textures: Some(tileset)
        };
    }
}