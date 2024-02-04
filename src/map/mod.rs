use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::Arc;

use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Commands, default, Entity, Event, EventReader, EventWriter, Query, Res, Resource, SpriteBundle, Transform, Vec2, Vec3, With};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

use crate::grid::resources::Grid;
use crate::map::resources::MapEntity;
use crate::map::systems::{map_save_system, save_directory_picked};
use crate::TextureResource;
use crate::tiles::{Tile, TileType};

pub(crate) mod systems;
pub(crate) mod resources;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, map_save_system);
        app.add_systems(Update, tile_spawn_reader);
        app.add_systems(Update, save_directory_picked);
        app.add_systems(Update, spawn_map_entity_reader);
        app.add_systems(Update, clear_tiles_reader);

        app.add_event::<TilePlaceEvent>();
        app.add_event::<SpawnMapEntity>();
        app.add_event::<ClearTiles>();
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
    pub fn set_tile_at(
        &mut self,
        cords: (i32, i32),
        tile_type: TileType,
        e_set_tile: &mut EventWriter<TilePlaceEvent>,
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
}

#[derive(Event)]
pub struct SpawnMapEntity {
    pub map_entity: Arc<MapEntity>,
}

pub fn spawn_map_entity_reader(
    mut e_spawn_map_entity: EventReader<SpawnMapEntity>,
    mut e_tile_place: EventWriter<TilePlaceEvent>,
) {
    for event in e_spawn_map_entity.read() {
        for (_, tile) in event.map_entity.tiles.tiles.iter() {
            e_tile_place.send(
                TilePlaceEvent {
                    tile: tile.clone()
                }
            )
        }
    }
}

#[derive(Event)]
pub struct ClearTiles;

pub fn clear_tiles_reader(
    mut q_tiles: Query<Entity, With<Tile>>,
    mut e_clear_tiles: EventReader<ClearTiles>,
    mut commands: Commands,
) {
    for _ in e_clear_tiles.read() {
        for entity in q_tiles.iter_mut() {
            let mut entity_commands = commands.get_entity(entity).unwrap();
            entity_commands.despawn();
        }
    }
}