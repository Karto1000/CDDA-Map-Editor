use std::fmt::Formatter;
use std::sync::Arc;

use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::prelude::{Commands, default, Entity, Event, EventReader, EventWriter, Image, Query, Res, Resource, SpriteBundle, Transform, Vec3, With};
use bevy::reflect::TypeData;
use image::imageops::tile;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

use crate::{EditorData, GraphicsResource};
use crate::graphics::SpriteType;
use crate::grid::resources::Grid;
use crate::map::map_entity::MapEntity;
use crate::map::systems::{map_save_system, save_directory_picked};
use crate::project::Project;
use crate::tiles::Tile;

pub(crate) mod systems;
pub(crate) mod map_entity;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, map_save_system);
        app.add_systems(Update, tile_spawn_reader);
        app.add_systems(Update, save_directory_picked);
        app.add_systems(Update, spawn_map_entity_reader);
        app.add_systems(Update, clear_tiles_reader);
        app.add_systems(Update, update_sprite_reader);

        app.add_event::<TilePlaceEvent>();
        app.add_event::<UpdateSpriteEvent>();
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

#[derive(Event, Debug)]
pub struct UpdateSpriteEvent {
    tile: Tile,
    entity: Entity,
}

#[derive(Event, Debug)]
pub struct TilePlaceEvent {
    tile: Tile,
}

pub fn update_sprite_reader(
    mut e_update_sprite: EventReader<UpdateSpriteEvent>,
    mut q_sprite: Query<&mut Handle<Image>, With<Tile>>,
    r_textures: Res<GraphicsResource>,
    r_editor_data: Res<EditorData>,
) {
    let project = match r_editor_data.get_current_project() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_update_sprite.read() {
        let sprite = get_fitting_sprite(&e.tile, &project, &r_textures);

        let mut image = q_sprite.get_mut(e.entity).unwrap();
        *image = sprite.clone();
    }
}

fn get_fitting_sprite<'a>(tile: &'a Tile, project: &'a Project, res_textures: &'a Res<GraphicsResource>) -> &'a Handle<Image> {
    let sprite_type = res_textures.get_texture(&project.map_entity.get_tile_id_from_character(&tile.character));

    return match sprite_type {
        SpriteType::Single(s) => s,
        SpriteType::Multitile { center, corner, t_connection, edge, end_piece, unconnected } => {
            let tiles_around = project.map_entity.get_tiles_around(&Coordinates { x: tile.x, y: tile.y });

            let is_tile_ontop_same_type = match tiles_around.0 {
                None => false,
                Some(top) => top.character == tile.character
            };

            let is_tile_right_same_type = match tiles_around.1 {
                None => false,
                Some(right) => right.character == tile.character
            };

            let is_tile_below_same_type = match tiles_around.2 {
                None => false,
                Some(below) => below.character == tile.character
            };

            let is_tile_left_same_type = match tiles_around.3 {
                None => false,
                Some(left) => left.character == tile.character
            };

            return match (is_tile_ontop_same_type, is_tile_right_same_type, is_tile_below_same_type, is_tile_left_same_type) {
                // Some of the worst code i've ever written lol
                (true, true, true, true) => &center,
                (true, true, true, false) => &t_connection.west,
                (true, true, false, true) => &t_connection.south,
                (true, false, true, true) => &t_connection.east,
                (false, true, true, true) => &t_connection.north,
                (true, true, false, false) => &corner.south_west,
                (true, false, false, true) => &corner.south_east,
                (false, true, true, false) => &corner.north_west,
                (false, false, true, true) => &corner.north_east,
                (true, false, false, false) => &end_piece.south,
                (false, true, false, false) => &end_piece.west,
                (false, false, true, false) => &end_piece.north,
                (false, false, false, true) => &end_piece.east,
                (false, true, false, true) => &edge.east_west,
                (true, false, true, false) => &edge.north_south,
                (false, false, false, false) => &unconnected
            };
        }
    };
}

pub fn tile_spawn_reader(
    mut commands: Commands,
    mut e_tile_place: EventReader<TilePlaceEvent>,
    mut e_update_sprite: EventWriter<UpdateSpriteEvent>,
    res_grid: Res<Grid>,
    res_textures: Res<GraphicsResource>,
    res_editor_data: Res<EditorData>,
) {
    let project = match res_editor_data.get_current_project() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_tile_place.read() {
        let sprite = get_fitting_sprite(&e.tile, &project, &res_textures);

        let tiles_around = project.map_entity.get_tiles_around(&Coordinates { x: e.tile.x, y: e.tile.y });

        commands.spawn((
            e.tile,
            SpriteBundle {
                texture: sprite.clone(),
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

#[derive(Event)]
pub struct SpawnMapEntity {
    pub map_entity: Arc<MapEntity>,
}

pub fn spawn_map_entity_reader(
    mut e_spawn_map_entity: EventReader<SpawnMapEntity>,
    mut e_tile_place: EventWriter<TilePlaceEvent>,
) {
    for event in e_spawn_map_entity.read() {
        for (_, tile) in event.map_entity.tiles.iter() {
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