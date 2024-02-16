use std::collections::HashMap;
use either::Either;
use rand::Rng;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::common::{ItemId, MeabyNumberRange, MeabyWeighted, TileId};
use crate::common::MeabyMulti;

pub(crate) mod loader;

fn default_chance() -> u32 {
    100
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Switch {
    pub param: String,
    pub fallback: Option<String>,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum MapGenValue {
    Simple(String),
    Distribution { distribution: Vec<MeabyWeighted<String>> },
    Param { param: String, fallback: Option<String> },
    Switch { switch: Switch, cases: HashMap<String, String> },
}

impl MapGenValue {
    pub fn get_value(&self) -> TileId {
        match self {
            MapGenValue::Simple(_) => { panic!() }
            MapGenValue::Distribution { distribution } => {
                let mut rng = rand::thread_rng();
                let random_index: usize = rng.gen_range(0..distribution.len());
                // TODO Take weights into account
                let random_id = distribution.get(random_index).unwrap();
                match random_id {
                    MeabyWeighted::Weighted(w) => {
                        return TileId(w.value.clone())
                    },
                    MeabyWeighted::NotWeighted(v) => return TileId(v.clone())
                };
            }
            MapGenValue::Param { .. } => { panic!() }
            MapGenValue::Switch { .. } => { panic!() }
        }
    }
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub enum ParameterType {
    #[serde(rename = "ter_str_id")]
    TerStrId,
    #[serde(rename = "furn_str_id")]
    FurnStrId,
    #[serde(rename = "nested_mapgen_id")]
    NestedMapgenId,
    #[serde(rename = "string")]
    // TODO: Figure out what this does
    String,
    #[serde(rename = "palette_id")]
    PaletteId,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    pub default: MapGenValue,

    #[serde(skip)]
    pub calculated_value: Option<TileId>
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct ParameterReference {
    pub param: String,
    pub fallback: String,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum Identifier {
    TileId(TileId),
    Parameter(ParameterReference),
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum MapObjectId {
    Grouped(Vec<MeabyWeighted<Identifier>>),
    Nested(Vec<Vec<MeabyWeighted<Identifier>>>),
    Param { param: String, fallback: Option<String> },
    Switch {
        switch: Switch,
        cases: HashMap<String, String>,
    },
    Single(MeabyWeighted<Identifier>),
}

#[derive(Deserialize, Clone, Serialize, Debug, Eq, Hash, PartialEq)]
pub struct ItemCollectionGroup {
    group: String,
    #[serde(default = "default_chance")]
    prob: u32,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct DistributionCollectionItem {
    item: String,
    count: MeabyNumberRange<i32>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct SubItem {
    item: String,
    #[serde(default = "default_chance")]
    prob: u32,
    count: Option<MeabyNumberRange<i32>>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct SubGroup {
    group: String,
    #[serde(default = "default_chance")]
    prob: u32,
    count: Option<MeabyNumberRange<i32>>,
}


#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum SubItemKind {
    Item(SubItem),
    Collection {
        collection: Vec<SubItem>,
        prob: u32,
    },
    Group(SubGroup),
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct SubType {
    subtype: String,
    entries: Vec<SubItemKind>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum Item {
    Default {
        item: ItemId,
        #[serde(default = "default_chance")]
        chance: u32,
        repeat: Option<MeabyNumberRange<u32>>,
    },
    Distribution {
        item: SubType,
        #[serde(default = "default_chance")]
        chance: u32,
        repeat: Option<MeabyNumberRange<u32>>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Palette {
    pub id: String,

    #[serde(default)]
    #[serde(skip_serializing)]
    pub parameters: HashMap<String, Parameter>,

    #[serde(default)]
    #[serde(skip_serializing)]
    pub terrain: HashMap<char, MapObjectId>,

    #[serde(default)]
    #[serde(skip_serializing)]
    pub furniture: HashMap<char, MapObjectId>,

    #[serde(default)]
    #[serde(skip_serializing)]
    pub items: HashMap<char, MeabyMulti<Item>>,

    #[serde(default)]
    #[serde(skip_serializing)]
    // TODO: Figure out what the value is here
    pub toilets: HashMap<char, Value>,
}

impl Default for Palette {
    fn default() -> Self {
        return Self {
            id: "unnamed".into(),
            parameters: HashMap::new(),
            terrain: HashMap::new(),
            furniture: HashMap::new(),
            items: HashMap::new(),
            toilets: HashMap::new(),
        };
    }
}