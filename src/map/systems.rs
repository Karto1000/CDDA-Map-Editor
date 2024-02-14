use std::fs;
use std::sync::Arc;

use bevy::asset::Handle;
use bevy::hierarchy::Children;
use bevy::input::Input;
use bevy::math::Vec3;
use bevy::prelude::{Commands, default, Entity, Event, EventReader, EventWriter, Image, KeyCode, Query, Res, ResMut, SpriteBundle, Text, Transform, With};
use bevy::text::TextSection;
use bevy_file_dialog::{DialogFileSaved, FileDialogExt};
use serde::{Deserialize, Serialize};

use crate::common::Coordinates;
use crate::EditorData;
use crate::graphics::{GraphicsResource, Sprite};
use crate::graphics::tileset::legacy::{GetBackground, GetForeground};
use crate::grid::resources::Grid;
use crate::map::{TileDeleteEvent, TilePlaceEvent};
use crate::map::events::{ClearTiles, SpawnMapEntity, UpdateSpriteEvent};
use crate::project::resources::{Project, ProjectSaveState};
use crate::tiles::components::Tile;
use crate::ui::tabs::components::Tab;

#[derive(Event)]
pub struct SpawnSprite {
    z: u32,
    tile: Tile,
    coordinates: Coordinates,
    sprite_kind: SpriteKind,
}

pub enum SpriteKind {
    Item(Sprite),
    Terrain(Sprite),
    Furniture(Sprite),
    Toilet(Sprite),
}

impl SpriteKind {
    pub fn get_fg(&self) -> &Option<Arc<dyn GetForeground>> {
        return match self {
            SpriteKind::Item(i) => &i.fg,
            SpriteKind::Terrain(t) => &t.fg,
            SpriteKind::Furniture(f) => &f.fg,
            SpriteKind::Toilet(t) => &t.fg
        };
    }

    pub fn get_bg(&self) -> &Option<Arc<dyn GetBackground>> {
        return match self {
            SpriteKind::Item(i) => &i.bg,
            SpriteKind::Terrain(t) => &t.bg,
            SpriteKind::Furniture(f) => &f.bg,
            SpriteKind::Toilet(t) => &t.bg
        };
    }
}

pub fn spawn_sprite(
    mut commands: Commands,
    r_grid: Res<Grid>,
    mut e_spawn_sprite: EventReader<SpawnSprite>,
    mut r_editor_data: ResMut<EditorData>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    for e in e_spawn_sprite.read() {
        let fg = e.sprite_kind.get_fg();
        let bg = e.sprite_kind.get_bg();

        if fg.is_some() {
            let fg_entity_commands = commands.spawn((
                e.tile.clone(),
                SpriteBundle {
                    texture: fg.as_ref().unwrap().get_sprite().clone(),
                    transform: Transform {
                        translation: Vec3 {
                            // Spawn off-screen
                            x: -1000.0,
                            y: -1000.0,
                            z: e.z as f32 + 1.,
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

            let tile = project.map_entity.tiles.get_mut(&e.coordinates).unwrap();
            match &e.sprite_kind {
                SpriteKind::Item(_) => { panic!("Not Implemented") }
                SpriteKind::Terrain(_) => {
                    tile.terrain.fg_entity = Some(fg_entity_commands.id());
                }
                SpriteKind::Furniture(_) => {
                    tile.furniture.fg_entity = Some(fg_entity_commands.id())
                }
                SpriteKind::Toilet(_) => { panic!("Not Implemented") }
            }
        }

        if bg.is_some() {
            let bg_entity_commands = commands.spawn((
                e.tile.clone(),
                SpriteBundle {
                    texture: bg.as_ref().unwrap().get_sprite().clone(),
                    transform: Transform {
                        translation: Vec3 {
                            // Spawn off screen
                            x: -1000.0,
                            y: -1000.0,
                            z: e.z as f32,
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

            let tile = project.map_entity.tiles.get_mut(&e.coordinates).unwrap();
            match &e.sprite_kind {
                SpriteKind::Item(_) => { panic!("Not Implemented") }
                SpriteKind::Terrain(_) => {
                    tile.terrain.bg_entity = Some(bg_entity_commands.id());
                }
                SpriteKind::Furniture(_) => {
                    tile.furniture.bg_entity = Some(bg_entity_commands.id())
                }
                SpriteKind::Toilet(_) => { panic!("Not Implemented") }
            };
        }
    }
}

pub fn map_save_system(
    keys: Res<Input<KeyCode>>,
    r_editor_data: ResMut<EditorData>,
    mut commands: Commands,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::S) {
        let current_project = r_editor_data.get_current_project().unwrap();

        match &current_project.save_state {
            ProjectSaveState::Saved(p) => {
                fs::write(p, serde_json::to_string(&current_project).unwrap().into_bytes()).unwrap();
            }
            _ => {
                commands.dialog()
                    .set_file_name("unnamed.map")
                    .save_file::<Project>(serde_json::to_string(&current_project).unwrap().into_bytes());
            }
        }
    }
}

pub fn save_directory_picked(
    mut res_editor_data: ResMut<EditorData>,
    mut e_file_saved: EventReader<DialogFileSaved<Project>>,
    q_tabs: Query<(Entity, &Tab, &Children)>,
    mut q_text: Query<&mut Text>,
) {
    let project_index = res_editor_data.current_project_index;
    let current_project = match res_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    for event in e_file_saved.read() {
        current_project.save_state = ProjectSaveState::Saved(event.path.clone());
        current_project.map_entity.map_type.set_name(event.path.file_name().unwrap().to_str().unwrap().to_string());

        // Edit the file name in the saved file because we can't know the file name in advance
        let content = fs::read_to_string(&event.path).unwrap();
        let mut entity: Project = serde_json::from_str(content.as_str()).unwrap();

        // This is probably some of the weirdest code i've ever written
        let file_name_string = event.path
            .file_name()
            .clone()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let reversed_string = file_name_string
            .chars()
            .rev()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("");

        let project_name = reversed_string
            // Remove the extension with the dot
            .splitn(2, ".")
            .last()
            .unwrap()
            .chars()
            .rev()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("");

        for (_, tab, children) in q_tabs.iter() {
            if tab.index != project_index { continue; }

            for child in children.iter() {
                let mut text = match q_text.get_mut(*child) {
                    Ok(t) => t,
                    Err(_) => { continue; }
                };

                text.sections.clear();
                text.sections.push(TextSection::from(project_name.clone()));
            }
        }

        current_project.map_entity.map_type.set_name(project_name);
        entity.save_state = ProjectSaveState::Saved(event.path.clone());

        // Remove the original file and Save it back and overwrite the original file
        fs::remove_file(&event.path).unwrap();
        fs::write(&event.path, serde_json::to_string(&entity).unwrap().into_bytes()).unwrap();
    }
}

pub fn set_tile_reader(
    mut e_set_tile: EventReader<TilePlaceEvent>,
    mut r_editor_data: ResMut<EditorData>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_set_tile.read() {
        project.map_entity.tiles.insert(
            e.coordinates.clone(),
            e.tile,
        );
    }
}

pub fn tile_remove_reader(
    mut e_delete_tile: EventReader<TileDeleteEvent>,
    mut r_editor_data: ResMut<EditorData>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_delete_tile.read() {
        project.map_entity.tiles.remove(&e.coordinates);
    }
}

pub fn update_sprite_reader(
    mut commands: Commands,
    mut e_update_sprite: EventReader<UpdateSpriteEvent>,
    mut q_sprite: Query<&mut Handle<Image>, With<Tile>>,
    mut r_editor_data: ResMut<EditorData>,
    r_textures: Res<GraphicsResource>,
    r_grid: Res<Grid>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_update_sprite.read() {
        let tile_sprite = r_textures.textures.get_textures(&project, &e.tile.character, &e.coordinates);
        macro_rules! spawn_sprite {
            ($sprite: expr, $tile_path: expr, $sprite_type: ident) => {
                if let Some(fg) = &$sprite.fg {
                    match $tile_path.fg_entity {
                        None => {
                            // Spawn the Sprite
                            let fg_entity_commands = commands.spawn((
                                e.tile,
                                SpriteBundle {
                                    texture: fg.get_sprite().clone(),
                                    transform: Transform {
                                         translation: Vec3 {
                                                       // Spawn off screen
                                                       x: -1000.0,
                                                       y: -1000.0,
                                                       z: 2.0,
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

                            let tile = project.map_entity.tiles.get_mut(&e.coordinates).unwrap();
                            tile.$sprite_type.fg_entity = Some(fg_entity_commands.id());
                        }
                        Some(i) => {
                            match q_sprite.get_mut(i) {
                               Ok(mut i) => {
                                   *i = fg.get_sprite().clone()
                               }
                                   Err(_) => {}
                               }
                        }
                    }
                }

                if let Some(bg) = &$sprite.bg {
                    match $tile_path.bg_entity {
                        None => {
                            let bg_entity_commands = commands.spawn((
                                e.tile,
                                SpriteBundle {
                                     texture: bg.get_sprite().clone(),
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

                            let tile = project.map_entity.tiles.get_mut(&e.coordinates).unwrap();
                            tile.$sprite_type.bg_entity = Some(bg_entity_commands.id());
                        }
                        Some(i) => match q_sprite.get_mut(i) {
                            Ok(mut i) => {
                                match $sprite.bg.as_ref() {
                                    None => {
                                      // Sprite was deleted
                                    }
                                    Some(s) => {
                                      *i = s.get_sprite().clone();
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        }

        if let Some(sprite) = tile_sprite.terrain {
            spawn_sprite!(sprite, e.tile.terrain, terrain);
        }

        if let Some(sprite) = tile_sprite.furniture {
            spawn_sprite!(sprite, e.tile.furniture, furniture);
        }
    }
}

pub fn tile_spawn_reader(
    mut commands: Commands,
    mut e_tile_place: EventReader<TilePlaceEvent>,
    mut e_spawn_sprite: EventWriter<SpawnSprite>,
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
        let sprites = r_textures.textures.get_textures(project, &e.tile.character, &e.coordinates);

        if let Some(terrain) = sprites.terrain {
            e_spawn_sprite.send(
                SpawnSprite {
                    coordinates: e.coordinates.clone(),
                    sprite_kind: SpriteKind::Terrain(terrain.clone()),
                    tile: e.tile.clone(),
                    z: 1
                }
            )
        }

        if let Some(furniture) = sprites.furniture {
            e_spawn_sprite.send(
                SpawnSprite {
                    coordinates: e.coordinates.clone(),
                    sprite_kind: SpriteKind::Furniture(furniture.clone()),
                    tile: e.tile.clone(),
                    z: 3
                }
            )
        }


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

    for e in e_tile_delete.read() {
        macro_rules! despawn {
            ($path: expr) => {
                match $path.fg_entity {
                    None => {}
                    Some(entity) => {
                        commands.get_entity(entity).unwrap().despawn()
                    }
                }

                match $path.bg_entity {
                    None => {}
                    Some(entity) => {
                        commands.get_entity(entity).unwrap().despawn()
                    }
                }
            }
        }

        despawn!(e.tile.terrain);
        despawn!(e.tile.furniture);
        despawn!(e.tile.toilets);
        despawn!(e.tile.items);

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