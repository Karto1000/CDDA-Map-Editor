use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, Resource};
use serde::{Deserialize, Serialize};

use crate::common::Coordinates;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct SpriteRepresentation {
    #[serde(skip)]
    pub fg_entity: Option<Entity>,

    #[serde(skip)]
    pub bg_entity: Option<Entity>,
}

#[derive(Default, Debug, Component, Clone)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl From<Coordinates> for Offset {
    fn from(value: Coordinates) -> Self {
        return Self {
            x: value.x,
            y: value.y,
        };
    }
}

impl Default for SpriteRepresentation {
    fn default() -> Self {
        return Self {
            fg_entity: None,
            bg_entity: None,
        };
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct Tile {
    pub character: char,

    #[serde(skip)]
    pub fallback: SpriteRepresentation,

    #[serde(skip)]
    pub terrain: SpriteRepresentation,

    #[serde(skip)]
    pub furniture: SpriteRepresentation,

    #[serde(skip)]
    pub items: SpriteRepresentation,

    #[serde(skip)]
    pub toilets: SpriteRepresentation,
    // TODO: Add missing representations
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        return Self {
            character: value,
            fallback: SpriteRepresentation::default(),
            terrain: SpriteRepresentation::default(),
            furniture: SpriteRepresentation::default(),
            items: SpriteRepresentation::default(),
            toilets: SpriteRepresentation::default(),
        };
    }
}

#[derive(Resource, Debug)]
pub struct PlaceInfo {
    pub last_place_position: Option<Vec2>,
}
