use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, DerefMut};
use std::sync::{Arc, RwLock};

use bevy::prelude::Component;
use bevy::prelude::Event;
use color_print::cformat;
use lazy_static::lazy_static;
use log::{Level, Log, Metadata, Record};
use num::Num;
use rand::{Rng, SeedableRng, thread_rng};
use rand::distributions::Distribution;
use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::distributions::WeightedIndex;
use rand::rngs::StdRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

pub(crate) mod io;

pub type TileId = String;

lazy_static! {
    pub static ref RANDOM: Arc<RwLock<StdRng>> = Arc::new(RwLock::new(StdRng::seed_from_u64(1)));
}

pub struct BufferedLogger {
    pub log_queue: Arc<RwLock<Vec<LogMessage>>>,
}

impl BufferedLogger {
    pub fn new() -> Self {
        return Self {
            log_queue: Arc::new(RwLock::new(Vec::new()))
        };
    }
}

impl Log for BufferedLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut guard = self.log_queue.write().unwrap();

        match record.level() {
            Level::Error => {
                println!("{}", cformat!("<r>[ERROR]</r> {}", record.args().to_string()))
            }
            Level::Warn => {
                println!("{}", cformat!("<y>[WARN]</y> {}", record.args().to_string()))
            }
            Level::Info => {
                println!("{}", cformat!("<b>[INFO]</b> {}", record.args().to_string()))
            }
            Level::Debug => {
                println!("{}", cformat!("<w>[DEBUG]</w> {}", record.args().to_string()))
            }
            Level::Trace => {}
        }
        guard.push(LogMessage::new(record.level(), record.args().to_string()));
    }

    fn flush(&self) {
        let mut guard = self.log_queue.write().unwrap();
        guard.clear();
    }
}

#[derive(Event, Debug)]
pub struct LogMessage {
    pub level: Level,
    pub message: String,
}

impl LogMessage {
    pub fn new(level: Level, msg: String) -> Self {
        return Self {
            level,
            message: msg,
        };
    }
    pub fn info(msg: String) -> Self {
        return Self {
            level: Level::Info,
            message: msg,
        };
    }

    pub fn warning(msg: String) -> Self {
        return Self {
            level: Level::Warn,
            message: msg,
        };
    }

    pub fn error(msg: String) -> Self {
        return Self {
            level: Level::Error,
            message: msg,
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MeabyMulti<T> {
    Multi(Vec<T>),
    Single(T),
}

impl<T> MeabyMulti<T> {
    pub fn multi(self) -> Vec<T> {
        return match self {
            MeabyMulti::Multi(mul) => mul,
            MeabyMulti::Single(_) => panic!("Tried to call 'multi()' on a Single MeabyMulti Variant")
        };
    }

    pub fn single(self) -> T {
        return match self {
            MeabyMulti::Multi(_) => panic!("Tried to call 'single()' on a Multi MeabyMulti Variant"),
            MeabyMulti::Single(s) => s
        };
    }
}


pub trait GetRandom<T> {
    fn get_random_weighted(&self) -> Option<&T>;
}

impl<T: Debug> GetRandom<T> for Vec<Weighted<T>> {
    fn get_random_weighted(&self) -> Option<&T> {
        if self.len() == 0 { return None; }

        let dist = WeightedIndex::new(self.iter().map(|w| {
            match w.weight {
                // TODO: Figure out what to do when all weights are 0
                0 => 1.,
                _ => w.weight as f32
            }
        }).collect::<Vec<f32>>().as_slice()).unwrap();
        let mut lock = RANDOM.write().unwrap();

        return match self.get(dist.sample(lock.deref_mut())) {
            None => None,
            Some(v) => Some(&v.value)
        };
    }
}

impl<T> GetRandom<T> for Vec<MeabyWeighted<T>> {
    fn get_random_weighted(&self) -> Option<&T> {
        if self.len() == 0 { return None; }

        let dist = WeightedIndex::new(self.iter().map(|mw| match mw {
            MeabyWeighted::NotWeighted(_) => 1.,
            MeabyWeighted::Weighted(w) => w.weight as f32
        }).collect::<Vec<f32>>().as_slice()).unwrap();
        let mut lock = RANDOM.write().unwrap();

        return match self.get(dist.sample(lock.deref_mut())) {
            None => None,
            Some(v) => match v {
                MeabyWeighted::NotWeighted(nw) => Some(nw),
                MeabyWeighted::Weighted(w) => Some(&w.value)
            }
        };
    }
}

impl<K> GetRandom<K> for HashMap<K, u32> {
    fn get_random_weighted(&self) -> Option<&K> {
        if self.is_empty() { return None; }

        let dist = WeightedIndex::new(self.values().map(|v| *v as f32).collect::<Vec<f32>>()).unwrap();

        let items = self.keys().collect::<Vec<&K>>();
        let mut lock = RANDOM.write().unwrap();

        return Some(items[dist.sample(lock.deref_mut())]);
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

impl<T> MeabyWeighted<T> {
    pub fn value(&self) -> &T {
        return match self {
            MeabyWeighted::NotWeighted(v) => v,
            MeabyWeighted::Weighted(w) => &w.value
        };
    }

    pub fn weighted(self) -> Weighted<T> {
        return match self {
            MeabyWeighted::NotWeighted(_) => panic!("Tried to call 'weighted()' on a NotWeighted MeabyWeighted Variant"),
            MeabyWeighted::Weighted(w) => w
        };
    }

    pub fn not_weighted(self) -> T {
        return match self {
            MeabyWeighted::NotWeighted(nw) => nw,
            MeabyWeighted::Weighted(_) => panic!("Tried to call 'not_weighted()' on a Weighted MeabyWeighted Variant")
        };
    }
}


pub struct CoordinatesVisitor;

impl<'de> Visitor<'de> for CoordinatesVisitor {
    type Value = Coordinates;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("an string of two numbers separated by a semicolon (example: 10;10)")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
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

impl Add for Coordinates {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        return Self::new(self.x + rhs.x, self.y + rhs.y);
    }
}

impl Serialize for Coordinates {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        return Ok(serializer.serialize_str(&format!("{};{}", self.x, self.y))?);
    }
}

impl<'de> Deserialize<'de> for Coordinates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        return Ok(deserializer.deserialize_str(CoordinatesVisitor)?);
    }
}

