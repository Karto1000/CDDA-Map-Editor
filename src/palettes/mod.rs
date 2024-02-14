use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::{ItemId, MeabyWeighted, TileId};
use crate::common::MeabyMulti;

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
    Param { param: String },
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct ItemGroup {
    group: String,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum Item {
    Single {
        item: ItemId,
        chance: u32,
        repeat: Option<(u32, u32)>,
    },
    Distribution {
        subtype: String,
        entries: Vec<ItemGroup>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Palette {
    pub id: String,
    pub terrain: HashMap<char, MapObjectId>,
    pub furniture: HashMap<char, MapObjectId>,

    #[serde(default)]
    pub items: HashMap<char, MeabyMulti<Item>>,

    #[serde(default)]
    // TODO: Figure out what the value is here
    pub toilets: HashMap<char, Value>,
}

impl Default for Palette {
    fn default() -> Self {
        return Self {
            id: "unnamed".into(),
            terrain: HashMap::new(),
            furniture: HashMap::new(),
            items: HashMap::new(),
            toilets: HashMap::new(),
        };
    }
}