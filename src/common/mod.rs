use std::collections::HashMap;
use std::fmt::Formatter;

use bevy::prelude::Color;
use bevy::prelude::Component;
use num::{Bounded, Num};
use rand::{Rng, thread_rng};
use rand::distributions::uniform::{SampleRange, SampleUniform};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

pub(crate) mod io;

pub const PRIMARY_COLOR: Color = Color::rgb(0.19, 0.21, 0.23);
pub const PRIMARY_COLOR_FADED: Color = Color::rgb(0.23, 0.25, 0.27);
pub const PRIMARY_COLOR_SELECTED: Color = Color::rgb(0.63, 0.70, 0.76);
pub const ERROR: Color = Color::rgba(0.79, 0.2, 0.21, 0.5);


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MeabyMulti<T> {
    Multi(Vec<T>),
    Single(T),
}

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug)]
pub struct TileId(pub String);

impl From<&'static str> for TileId {
    fn from(value: &'static str) -> Self {
        return Self { 0: value.to_string() };
    }
}

pub trait GetRandom<T> {
    fn get_random_weighted(&self) -> Option<&T>;
}

impl<T> GetRandom<T> for Vec<Weighted<T>> {
    fn get_random_weighted(&self) -> Option<&T> {
        let mut rng = thread_rng();
        let random_index: usize = rng.gen_range(0..self.len());
        // TODO Take weights into account
        let weighted = self.get(random_index).unwrap();
        return Some(&weighted.value);
    }
}

impl<T> GetRandom<T> for Vec<MeabyWeighted<T>> {
    fn get_random_weighted(&self) -> Option<&T> {
        let mut rng = thread_rng();
        let random_index: usize = rng.gen_range(0..self.len());
        // TODO Take weights into account
        let random_id = self.get(random_index).unwrap();

        return match random_id {
            MeabyWeighted::Weighted(w) => Some(&w.value),
            MeabyWeighted::NotWeighted(v) => Some(&v)
        };
    }
}

impl<K> GetRandom<K> for HashMap<K, u32> {
    fn get_random_weighted(&self) -> Option<&K> {
        if self.is_empty() { return None; }

        let mut rng = thread_rng();
        let keys: Vec<&K> = self.keys().collect();
        let random_index = rng.gen_range(0..keys.len());

        // TODO Take weights into account
        return Some(keys.get(random_index).unwrap().clone());
    }
}

#[derive(Serialize, Deserialize, Eq, Hash, PartialEq, Clone, Debug)]
pub struct ItemId(pub String);

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Weighted<T> {
    #[serde(alias = "value", alias = "sprite")]
    pub value: T,
    pub weight: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MeabyNumberRange<T: Num + SampleUniform + PartialOrd + Clone> {
    Range(T, T),
    Single(T),
}

impl<T: Num + SampleUniform + PartialOrd + Clone> MeabyNumberRange<T> {
    pub fn get_num(&self) -> T {
        return match self {
            MeabyNumberRange::Range(start, end) => {
                let mut rng = thread_rng();
                return rng.gen_range(start.clone()..end.clone()).clone();
            }
            MeabyNumberRange::Single(num) => num.clone()
        };
    }
}

impl<T> Weighted<T> {
    pub fn new(value: T, weight: u32) -> Self {
        return Self {
            value,
            weight,
        };
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    NotWeighted(T),
    Weighted(Weighted<T>),
}


pub struct CoordinatesVisitor;

impl<'de> Visitor<'de> for CoordinatesVisitor {
    type Value = Coordinates;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an string of two numbers separated by a semicolon (example: 10;10)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
        let split: Vec<&str> = v.split(";").collect::<Vec<&str>>();

        return Ok(Coordinates {
            x: split.get(0).expect("Value before ';'").parse().expect("Valid i32"),
            y: split.get(1).expect("Value after ';'").parse().expect("Valid i32"),
        });
    }
}

#[derive(Component, Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Coordinates {
    pub fn new(x: i32, y: i32) -> Self {
        return Self {
            x,
            y,
        };
    }
}

impl Serialize for Coordinates {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        return Ok(serializer.serialize_str(&format!("{};{}", self.x, self.y))?);
    }
}

impl<'de> Deserialize<'de> for Coordinates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        return Ok(deserializer.deserialize_str(CoordinatesVisitor)?);
    }
}

