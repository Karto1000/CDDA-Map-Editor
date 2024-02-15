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
