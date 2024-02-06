use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub(crate) mod loader;

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Weighted<T> {
    value: T,
    weight: u32,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    NotWeighted(T),
    Weighted(Weighted<T>),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Switch {
    param: String,
    fallback: Option<String>,
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
    parameter_type: String,
    default: MapGenValue,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum MapObjectId {
    Single(MeabyWeighted<String>),
    Grouped(Vec<MeabyWeighted<String>>),
    Nested(Vec<Vec<MeabyWeighted<String>>>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Palette {
    id: String,
    terrain: HashMap<char, MapObjectId>,
    furniture: HashMap<char, MapObjectId>,
}