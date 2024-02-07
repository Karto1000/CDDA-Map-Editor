use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

pub const PRIMARY_COLOR: Color = Color::rgb(0.19, 0.21, 0.23);
pub const PRIMARY_COLOR_FADED: Color = Color::rgb(0.23, 0.25, 0.27);
pub const PRIMARY_COLOR_SELECTED: Color = Color::rgb(0.63, 0.70, 0.76);
pub const ERROR: Color = Color::rgba(0.79, 0.2, 0.21, 0.5);

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug)]
pub struct TileId(pub String);

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Weighted<T> {
    #[serde(alias = "value", alias = "sprite")]
    pub value: T,
    pub weight: u32,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    NotWeighted(T),
    Weighted(Weighted<T>),
}