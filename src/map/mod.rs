use bevy::app::{App, Plugin, Update};
use bevy::asset::Handle;
use bevy::prelude::{Commands, Component, default, Entity, Event, EventReader, EventWriter, Image, Query, Res, ResMut, Resource, SpriteBundle, Transform, Vec3, With};
use bevy::prelude::IntoSystemConfigs;
use bevy::reflect::TypeData;
use bevy::utils::tracing::Instrument;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

use crate::{EditorData, GraphicsResource};
use crate::graphics::{GetTexture, LegacyTextures};
use crate::grid::resources::Grid;
use crate::map::events::{ClearTiles, SpawnMapEntity, TileDeleteEvent, TilePlaceEvent, UpdateSpriteEvent};
use crate::map::systems::{map_save_system, save_directory_picked};
use crate::tiles::components::Tile;

pub(crate) mod systems;
pub(crate) mod resources;
pub(crate) mod events;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, map_save_system);
        app.add_systems(Update, save_directory_picked);
        app.add_systems(Update, spawn_map_entity_reader);
        app.add_systems(Update, clear_tiles_reader);
        app.add_systems(Update, update_sprite_reader);

        app.add_event::<TilePlaceEvent>();
        app.add_event::<TileDeleteEvent>();
        app.add_event::<UpdateSpriteEvent>();
        app.add_event::<SpawnMapEntity>();
        app.add_event::<ClearTiles>();
    }
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
        let sprite = r_textures.textures.get_texture(&project, &e.tile.character, &e.coordinates);

        let mut image = match q_sprite.get_mut(e.tile.entity.unwrap()) {
            Ok(i) => { i }
            Err(_) => { return; }
        };
        *image = sprite.clone();
    }
}

pub fn tile_spawn_reader(
    mut commands: Commands,
    mut e_tile_place: EventReader<TilePlaceEvent>,
    mut e_update_sprite: EventWriter<UpdateSpriteEvent>,
    r_grid: Res<Grid>,
    r_textures: Res<GraphicsResource>,
    mut r_editor_data: ResMut<EditorData>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_tile_place.read() {
        let sprite = r_textures.textures.get_texture(&project, &e.tile.character, &e.coordinates);

        let entity_commands = commands.spawn((
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
                        x: r_grid.tile_size / r_grid.default_tile_size,
                        y: r_grid.tile_size / r_grid.default_tile_size,
                        z: 0.,
                    },
                    ..default()
                },
                ..default()
            },
            e.coordinates.clone()
        ));

        project.map_entity.tiles.get_mut(&e.coordinates).unwrap().entity = Some(entity_commands.id());

        // Check here because i couldn't figure out why the sprites were not correct when spawning a saved map
        if e.should_update_sprites {
            let tiles_around = project.map_entity.get_tiles_around(&e.coordinates);

            for (tile, coordinates) in tiles_around {
                match tile {
                    None => {}
                    Some(t) => {
                        e_update_sprite.send(
                            UpdateSpriteEvent {
                                tile: *t,
                                coordinates,
                            }
                        )
                    }
                }
            }
        }
    }
}

pub fn tile_despawn_reader(
    mut commands: Commands,
    mut e_tile_delete: EventReader<TileDeleteEvent>,
    mut e_update_sprite: EventWriter<UpdateSpriteEvent>,
    r_editor_data: Res<EditorData>,
) {
    let project = match r_editor_data.get_current_project() {
        None => { return; }
        Some(p) => { p }
    };

    for event in e_tile_delete.read() {
        match event.tile.entity {
            None => {}
            Some(e) => {
                let tiles_around = project.map_entity.get_tiles_around(&event.coordinates);

                for (tile, coordinates) in tiles_around {
                    match tile {
                        None => {}
                        Some(t) => {
                            e_update_sprite.send(
                                UpdateSpriteEvent {
                                    tile: *t,
                                    coordinates,
                                }
                            )
                        }
                    }
                }

                commands.get_entity(e).unwrap().despawn()
            }
        }
    }
}

pub fn spawn_map_entity_reader(
    mut e_spawn_map_entity: EventReader<SpawnMapEntity>,
    mut e_tile_place: EventWriter<TilePlaceEvent>,
) {
    for event in e_spawn_map_entity.read() {
        for (coords, tile) in event.map_entity.tiles.iter() {
            e_tile_place.send(
                TilePlaceEvent {
                    tile: tile.clone(),
                    coordinates: coords.clone(),
                    should_update_sprites: false,
                }
            )
        }
    }
}

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