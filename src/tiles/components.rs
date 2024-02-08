use bevy::prelude::{Component, Entity};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component, PartialEq)]
pub struct Tile {
    pub character: char,
    pub entity: Option<Entity>,
}

