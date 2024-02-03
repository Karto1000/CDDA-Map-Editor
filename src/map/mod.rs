use std::collections::HashMap;
use std::error::Error;
use std::fmt::Formatter;

use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Commands, default, Event, EventReader, EventWriter, IVec2, Res, Resource, SpriteBundle, Transform, Vec2, Vec3};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;
use serde_json::Value;

use crate::grid::resources::Grid;
use crate::map::system::{map_save_system, save_directory_picked};
use crate::TextureResource;
use crate::tiles::{Tile, TileType};

pub(crate) mod system;
pub(crate) mod resources;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, map_save_system);
        app.add_systems(Update, tile_spawn_reader);
        app.add_systems(Update, save_directory_picked);

        app.add_event::<TilePlaceEvent>();
    }
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
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

#[derive(Serialize, Deserialize, Debug, Resource, Clone, Default)]
pub struct Tiles {
    pub size: Vec2,
    pub tiles: HashMap<Coordinates, Tile>,
}

#[derive(Event, Debug)]
pub struct TilePlaceEvent {
    tile: Tile,
}

pub fn tile_spawn_reader(
    mut commands: Commands,
    mut e_tile_place: EventReader<TilePlaceEvent>,
    res_grid: Res<Grid>,
    res_textures: Res<TextureResource>,
) {
    for e in e_tile_place.read() {
        commands.spawn((
            e.tile,
            SpriteBundle {
                texture: res_textures.textures.get(&e.tile.tile_type).unwrap().clone(),
                transform: Transform {
                    translation: Vec3 {
                        // Spawn off screen
                        x: -1000.0,
                        y: -1000.0,
                        z: 1.0,
                    },
                    scale: Vec3 {
                        x: res_grid.tile_size / res_grid.default_tile_size,
                        y: res_grid.tile_size / res_grid.default_tile_size,
                        z: 0.,
                    },
                    ..default()
                },
                ..default()
            },
        ));
    }
}

impl Tiles {
    pub fn export(&self) -> Result<Value, anyhow::Error> {
        let mut rows: Vec<String> = vec![];

        for _ in 0..self.size.y as i32 {
            let mut row = String::with_capacity(self.size.x as usize);
            (0..self.size.x as i32).for_each(|i| {
                row.insert(i as usize, " ".parse::<char>().unwrap())
            });
            rows.push(row);
        };

        return Ok(Value::Array(rows.iter().map(|e| Value::String(e.clone())).collect()));
    }

    pub fn set_tile_at(
        &mut self,
        cords: (i32, i32),
        tile_type: TileType,
        mut e_set_tile: &mut EventWriter<TilePlaceEvent>,
    ) {
        let coordinates = Coordinates { x: cords.0, y: cords.1 };
        if self.tiles.get(&coordinates).is_some() { return; }

        let tile = Tile { tile_type, x: cords.0, y: cords.1 };

        e_set_tile.send(TilePlaceEvent { tile });

        self.tiles.insert(
            coordinates,
            tile,
        );
    }

    pub fn spawn(&mut self, e_set_tile: &mut EventWriter<TilePlaceEvent>) {
        let tile_clone = self.tiles.clone();

        for cords in tile_clone.keys().into_iter() {
            let tile = self.tiles.get(cords).unwrap();
            e_set_tile.send(TilePlaceEvent { tile: *tile })
        }
    }

    pub fn load(&mut self, e_set_tile: &mut EventWriter<TilePlaceEvent>, entity: &Tiles) {
        for (cords, tile) in entity.tiles.iter() {
            self.set_tile_at(
                (cords.x, cords.y),
                tile.tile_type,
                e_set_tile,
            );
        }
    }

    pub fn get_size(&self) -> Option<IVec2> {
        let mut keys_sorted_x: Vec<Coordinates> = self.tiles.clone().into_keys().collect();
        let mut keys_sorted_y: Vec<Coordinates> = self.tiles.clone().into_keys().collect();

        keys_sorted_x.sort_by(|coords1, coords2| coords1.x.cmp(&coords2.x));
        keys_sorted_y.sort_by(|coords1, coords2| coords1.y.cmp(&coords2.y));

        let leftmost_tile = keys_sorted_x.first().cloned().unwrap();
        let rightmost_tile = keys_sorted_x.last().cloned().unwrap();

        let topmost_tile = keys_sorted_y.first().cloned().unwrap();
        let bottommost_tile = keys_sorted_y.last().cloned().unwrap();

        return Some(IVec2::new((rightmost_tile.x - leftmost_tile.x).abs() + 1, (bottommost_tile.y - topmost_tile.y).abs() + 1));
    }
}