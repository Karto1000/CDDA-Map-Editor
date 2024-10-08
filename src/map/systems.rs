use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;

use bevy::asset::{Assets, Handle};
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Commands, Component, Cuboid, default, Entity, Event, EventReader, EventWriter, Image, Mesh, Meshable, Query, Res, ResMut, SpriteBundle, State, Transform, With};
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::{PrimaryWindow, Window};
use log::warn;

use crate::common::Coordinates;
use crate::graphics::{GetTexture, GraphicsResource, Sprite, SpriteState, TileSprite};
use crate::graphics::tileset::{GetBackground, GetForeground};
use crate::map::data::{ClearTiles, SpawnMapEntity, TileDeleteEvent, TilePlaceEvent, UpdateSpriteEvent};
use crate::program::data::{OpenedProject, Program, ProgramState};
use crate::tiles::data::{Offset, Tile};
use crate::ui::grid::GridMaterial;
use crate::ui::grid::resources::Grid;

#[derive(Event)]
pub struct SpawnSprite {
    z: u32,
    tile: Tile,
    coordinates: Coordinates,
    sprite_kind: SpriteKind,
    offset: Offset,
}

pub enum SpriteKind {
    Item(Sprite),
    Terrain(Sprite),
    Furniture(Sprite),
    Toilet(Sprite),
    Fallback(Sprite),
}

#[derive(Component, Debug)]
pub struct Animated {
    cooldown: u16,
    last_update: u64,
}

impl SpriteKind {
    pub fn get_fg(&self) -> &Option<Arc<dyn GetForeground>> {
        return match self {
            SpriteKind::Item(i) => &i.fg,
            SpriteKind::Terrain(t) => &t.fg,
            SpriteKind::Furniture(f) => &f.fg,
            SpriteKind::Toilet(t) => &t.fg,
            SpriteKind::Fallback(f) => &f.fg
        };
    }

    pub fn get_bg(&self) -> &Option<Arc<dyn GetBackground>> {
        return match self {
            SpriteKind::Item(i) => &i.bg,
            SpriteKind::Terrain(t) => &t.bg,
            SpriteKind::Furniture(f) => &f.bg,
            SpriteKind::Toilet(t) => &t.bg,
            SpriteKind::Fallback(_) => &None
        };
    }
}

#[derive(Component, Debug)]
pub struct Layer(f32);

pub fn spawn_sprite(
    mut commands: Commands,
    r_grid: Res<Grid>,
    mut e_spawn_sprite: EventReader<SpawnSprite>,
    mut r_editor_data: ResMut<Program>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let project = match r_editor_data.projects.get_mut(index) {
        None => return,
        Some(p) => p
    };

    for e in e_spawn_sprite.read() {
        let fg = e.sprite_kind.get_fg();
        let bg = e.sprite_kind.get_bg();

        if fg.is_some() {
            let layer = e.z as f32 + 1. + e.coordinates.y as f32 * 10.;
            let mut fg_entity_commands = commands.spawn((
                e.tile.clone(),
                SpriteBundle {
                    texture: fg.as_ref().unwrap().get_randomized_sprite().clone(),
                    transform: Transform {
                        translation: Vec3 {
                            // Spawn off-screen
                            x: -1000.0,
                            y: -1000.0,
                            z: layer,
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
                e.coordinates.clone(),
                Layer(layer),
                Offset::from(e.offset.clone())
            ));

            let tile = project.map_entity.tiles_mut().get_mut(&e.coordinates).unwrap();
            match &e.sprite_kind {
                SpriteKind::Item(_) => { panic!("Not Implemented") }
                SpriteKind::Terrain(sprite) => {
                    if sprite.is_animated {
                        fg_entity_commands.insert(Animated { cooldown: 1, last_update: (chrono::prelude::Utc::now().timestamp_millis() / 1000) as u64 });
                    }
                    tile.terrain.fg_entity = Some(fg_entity_commands.id());
                }
                SpriteKind::Furniture(_) => {
                    tile.furniture.fg_entity = Some(fg_entity_commands.id())
                }
                SpriteKind::Toilet(_) => { panic!("Not Implemented") }
                SpriteKind::Fallback(_) => {
                    tile.fallback.fg_entity = Some(fg_entity_commands.id());
                }
            }
        }

        if bg.is_some() {
            let bg_entity_commands = commands.spawn((
                e.tile.clone(),
                SpriteBundle {
                    texture: bg.as_ref().unwrap().get_randomized_sprite().clone(),
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
                e.coordinates.clone(),
                Layer(e.z as f32),
                Offset::from(e.offset.clone())
            ));

            let tile = project.map_entity.tiles_mut().get_mut(&e.coordinates).unwrap();
            match &e.sprite_kind {
                SpriteKind::Item(_) => { panic!("Not Implemented") }
                SpriteKind::Terrain(_) => {
                    tile.terrain.bg_entity = Some(bg_entity_commands.id());
                }
                SpriteKind::Furniture(_) => {
                    tile.furniture.bg_entity = Some(bg_entity_commands.id())
                }
                SpriteKind::Toilet(_) => { panic!("Not Implemented") }
                SpriteKind::Fallback(_) => { panic!("Not Implemented") }
            };
        }
    }
}

pub fn update_animated_sprites(
    q_sprites: Query<(Entity, &Coordinates, &Animated, &Layer)>,
    mut commands: Commands,
    r_textures: Res<GraphicsResource>,
    r_grid: Res<Grid>,
    mut r_program: ResMut<Program>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let cdda_data = match &r_program.config.cdda_data {
        None => return,
        Some(d) => d
    };

    let current_project = match r_program.projects.get(index) {
        None => return,
        Some(p) => p
    };

    let textures = match &r_textures.textures {
        None => return,
        Some(t) => t
    };

    let mut fg_entities_to_set = HashMap::new();

    for (entity, cords, animated, layer) in q_sprites.iter() {
        let tile = match current_project.map_entity.tiles().get(cords) {
            None => {
                warn!("Tile at cords {:?} does not exist even though query matched tile", cords);
                continue;
            }
            Some(t) => t
        };
        match textures.get_terrain(current_project, &cdda_data, &tile.character, cords) {
            SpriteState::Defined(terrain) => {
                if (chrono::prelude::Utc::now().timestamp_millis() / 1000) as u64 - animated.last_update < animated.cooldown as u64 {
                    return;
                }

                let fg = terrain.fg.as_ref().unwrap().get_randomized_sprite();

                let mut entity_commands = commands.get_entity(entity).unwrap();
                let fg_entity_commands = entity_commands
                    .insert(
                        SpriteBundle {
                            texture: fg.clone(),
                            transform: Transform {
                                translation: Vec3 {
                                    // Spawn off-screen
                                    x: -1000.0,
                                    y: -1000.0,
                                    // TODO FIX
                                    z: 5. as f32 + 1. + cords.y as f32 * 10.,
                                },
                                scale: Vec3 {
                                    x: r_grid.tile_size / r_grid.default_tile_size,
                                    y: r_grid.tile_size / r_grid.default_tile_size,
                                    z: layer.0,
                                },
                                ..default()
                            },
                            ..default()
                        },
                    )
                    .remove::<Animated>()
                    .insert(Animated {
                        cooldown: animated.cooldown,
                        last_update: (chrono::prelude::Utc::now().timestamp_millis() / 1000) as u64,
                    });

                fg_entities_to_set.insert(cords, fg_entity_commands.id());
            }
            SpriteState::TextureNotFound => {}
            SpriteState::NotMapped => {}
        }
    }

    let current_project_mut = match r_program.projects.get_mut(index) {
        None => return,
        Some(p) => p
    };

    for (cords, entity) in fg_entities_to_set.into_iter() {
        current_project_mut.map_entity.tiles_mut().get_mut(cords).unwrap().terrain.fg_entity = Some(entity);
    }
}

pub fn set_tile_reader(
    mut e_set_tile: EventReader<TilePlaceEvent>,
    mut r_program: ResMut<Program>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let project = match r_program.projects.get_mut(index) {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_set_tile.read() {
        project.map_entity.tiles_mut().insert(
            e.coordinates.clone(),
            e.tile,
        );
    }
}

pub fn tile_remove_reader(
    mut e_delete_tile: EventReader<TileDeleteEvent>,
    mut r_program: ResMut<Program>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let project = match r_program.projects.get_mut(index) {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_delete_tile.read() {
        project.map_entity.tiles_mut().remove(&e.coordinates);
    }
}

pub fn update_sprite_reader(
    mut commands: Commands,
    mut e_update_sprite: EventReader<UpdateSpriteEvent>,
    mut q_sprite: Query<&mut Handle<Image>, With<Tile>>,
    mut r_program: ResMut<Program>,
    r_textures: Res<GraphicsResource>,
    r_grid: Res<Grid>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let cdda_data = match r_program.config.cdda_data.clone() {
        None => return,
        Some(d) => d
    };

    let project = match r_program.projects.get_mut(index) {
        None => { return; }
        Some(p) => { p }
    };

    let textures = match &r_textures.textures {
        None => return,
        Some(t) => t
    };

    for e in e_update_sprite.read() {
        let tile_sprite = textures.get_textures(&project, &cdda_data, &e.tile.character, &e.coordinates);

        macro_rules! spawn_sprite {
            ($sprite: expr, $tile_path: expr, $sprite_type: ident) => {
                if let Some(fg) = &$sprite.fg {
                    match $tile_path.fg_entity {
                        None => {
                            // Spawn the Sprite
                            let fg_entity_commands = commands.spawn((
                                e.tile,
                                SpriteBundle {
                                    texture: fg.get_randomized_sprite().clone(),
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
                                Animated {
                                    cooldown: 1,
                                    last_update: (chrono::prelude::Utc::now().timestamp_millis() / 1000) as u64 
                                },
                                e.coordinates.clone(),
                                Offset {x: $sprite.offset_x, y: $sprite.offset_y }
                            ));

                            let tile = project.map_entity.tiles_mut().get_mut(&e.coordinates).unwrap();
                            tile.$sprite_type.fg_entity = Some(fg_entity_commands.id());
                        }
                        Some(i) => {
                            match q_sprite.get_mut(i) {
                               Ok(mut i) => {
                                   *i = fg.get_randomized_sprite().clone()
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
                                     texture: bg.get_randomized_sprite().clone(),
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
                                e.coordinates.clone(),
                                Offset {x: $sprite.offset_x, y: $sprite.offset_y }
                            ));

                            let tile = project.map_entity.tiles_mut().get_mut(&e.coordinates).unwrap();
                            tile.$sprite_type.bg_entity = Some(bg_entity_commands.id());
                        }
                        Some(i) => match q_sprite.get_mut(i) {
                            Ok(mut i) => {
                                match $sprite.bg.as_ref() {
                                    None => {
                                      // Sprite was deleted
                                    }
                                    Some(s) => {
                                      *i = s.get_randomized_sprite().clone();
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            }
        }

        match tile_sprite {
            TileSprite::Exists { terrain, furniture, .. } => {
                if let Some(sprite) = terrain {
                    spawn_sprite!(sprite, e.tile.terrain, terrain);
                }

                if let Some(sprite) = furniture {
                    spawn_sprite!(sprite, e.tile.furniture, furniture);
                }
            }
            TileSprite::Fallback(default) => {
                spawn_sprite!(default, e.tile.fallback, fallback);
            }
            TileSprite::Empty => {}
        }
    }
}

pub fn tile_spawn_reader(
    mut e_tile_place: EventReader<TilePlaceEvent>,
    mut e_spawn_sprite: EventWriter<SpawnSprite>,
    mut e_update_sprite: EventWriter<UpdateSpriteEvent>,
    r_textures: Res<GraphicsResource>,
    mut r_program: ResMut<Program>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let cdda_data = match r_program.config.cdda_data.clone() {
        None => return,
        Some(d) => d
    };

    let project = match r_program.projects.get_mut(index) {
        None => { return; }
        Some(p) => { p }
    };

    let textures = match &r_textures.textures {
        None => return,
        Some(t) => t
    };

    for e in e_tile_place.read() {
        let sprites = textures.get_textures(project, &cdda_data, &e.tile.character, &e.coordinates);

        match sprites {
            TileSprite::Exists { terrain, furniture, .. } => {
                if let Some(terrain) = terrain {
                    e_spawn_sprite.send(
                        SpawnSprite {
                            coordinates: e.coordinates.clone(),
                            sprite_kind: SpriteKind::Terrain(terrain.clone()),
                            tile: e.tile.clone(),
                            z: 1,
                            offset: Offset { x: terrain.offset_x, y: terrain.offset_y },
                        }
                    );
                }

                if let Some(furniture) = furniture {
                    e_spawn_sprite.send(
                        SpawnSprite {
                            coordinates: e.coordinates.clone(),
                            sprite_kind: SpriteKind::Furniture(furniture.clone()),
                            tile: e.tile.clone(),
                            z: 3,
                            offset: Offset { x: furniture.offset_x, y: furniture.offset_y },
                        }
                    );
                }
            }
            TileSprite::Fallback(default) => {
                e_spawn_sprite.send(
                    SpawnSprite {
                        coordinates: e.coordinates.clone(),
                        sprite_kind: SpriteKind::Fallback(default.clone()),
                        tile: e.tile.clone(),
                        z: 1,
                        offset: Offset::default(),
                    }
                );
            }
            TileSprite::Empty => {}
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
                        );
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
    r_program: Res<Program>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let project = match r_program.projects.get(index) {
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
        despawn!(e.tile.fallback);

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
                    );
                }
            }
        }
    }
}

pub fn spawn_map_entity_reader(
    mut e_spawn_map_entity: EventReader<SpawnMapEntity>,
    mut e_tile_place: EventWriter<TilePlaceEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<GridMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut r_grid: ResMut<Grid>,
    r_program: Res<Program>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    for event in e_spawn_map_entity.read() {
        if let None = r_grid.instantiated_grid {
            let index = match q_opened_project.iter().next() {
                None => return,
                Some(o) => o.1.index
            };

            let window = q_windows.single();

            r_grid.instantiated_grid = Some(commands.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle::from(meshes.add(Cuboid::new(1., 1., 0.0).mesh())),
                    transform: Transform::from_scale(Vec3::new(window.width(), window.height(), 0.0)),
                    material: materials.add(GridMaterial {
                        tile_size: r_grid.tile_size,
                        offset: Vec2::default(),
                        mouse_pos: Default::default(),
                        is_cursor_captured: 0,
                        map_size: r_program.projects.get(index).unwrap().map_entity.size(),
                        scale_factor: 1.,
                        inside_grid_color: r_program.config.style.gray_light.rgb_to_vec3(),
                        outside_grid_color: r_program.config.style.gray_darker.rgb_to_vec3(),
                    }),
                    ..default()
                },
                crate::ui::grid::GridMarker {}
            )).id());
        }

        for (coords, tile) in event.map_entity.tiles().iter() {
            e_tile_place.send(
                TilePlaceEvent {
                    tile: tile.clone(),
                    coordinates: coords.clone(),
                    should_update_sprites: false,
                }
            );
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