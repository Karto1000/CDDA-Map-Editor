use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct Tile {
    pub character: char,

    #[serde(skip)]
    pub fg_entity: Option<Entity>,

    #[serde(skip)]
    pub bg_entity: Option<Entity>,
}

impl From<char> for Tile {
    fn from(value: char) -> Self {
        return Self {
            character: value,
            fg_entity: None,
            bg_entity: None,
        };
    }
}
