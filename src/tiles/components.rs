use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct SpriteRepresentation {
    #[serde(skip)]
    pub fg_entity: Option<Entity>,

    #[serde(skip)]
    pub bg_entity: Option<Entity>,
}

impl Default for SpriteRepresentation {
    fn default() -> Self {
        return Self {
            fg_entity: None,
            bg_entity: None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct Tile {
    pub character: char,

    pub terrain: SpriteRepresentation,
    pub furniture: SpriteRepresentation,
    pub items: SpriteRepresentation,
    pub toilets: SpriteRepresentation
    // TODO: Add missing representations
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        return Self {
            character: value,
            terrain: SpriteRepresentation::default(),
            furniture: SpriteRepresentation::default(),
            items: SpriteRepresentation::default(),
            toilets: SpriteRepresentation::default()
        };
    }
}
