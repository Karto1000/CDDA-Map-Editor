use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use crate::common::{MeabyWeighted, TileId};

pub(crate) mod loader;

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Switch {
    pub param: String,
    pub fallback: Option<String>,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum MapGenValue {
    Simple(String),
    Distribution { distribution: Vec<Vec<(String, u32)>> },
    Param { param: String, fallback: Option<String> },
    Switch { switch: Switch, cases: HashMap<String, String> },
}


#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: String,
    pub default: MapGenValue,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum MapObjectId {
    Single(MeabyWeighted<TileId>),
    Grouped(Vec<MeabyWeighted<TileId>>),
    Nested(Vec<Vec<MeabyWeighted<TileId>>>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Palette {
    pub id: String,
    pub terrain: HashMap<char, MapObjectId>,
    pub furniture: HashMap<char, MapObjectId>,
}