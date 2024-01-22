use std::collections::HashMap;

use bevy::prelude::{Commands, Component, default, Entity, Handle, Image, Res, Resource, SpriteBundle, Transform, Vec3};
use bevy::window::Window;
use serde::{Deserialize, Serialize};

use crate::Grid;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Component)]
pub struct Tile {
    pub tile_type: TileType,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum TileType {
    Test
}

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct MapEntity {
    pub name: String,
    pub weight: u32,
    pub tiles: HashMap<(i32, i32), Tile>,

    #[serde(skip)]
    pub texture: Handle<Image>,
}

impl MapEntity {
    pub fn get_size(&self) -> Option<(i32, i32)> {
        let mut keys: Vec<(i32, i32)> = self.tiles.clone().into_keys().collect();
        keys.sort();
        return keys.last().cloned();
    }

    pub fn new(name: String, texture: Handle<Image>) -> Self {
        return Self {
            name,
            weight: 100,
            tiles: HashMap::new(),
            texture,
        };
    }

    pub fn set_tile_at(
        &mut self,
        commands: &mut Commands,
        cords: (i32, i32),
        tile_type: TileType,
        res_grid: &Res<Grid>,
        window: &Window
    ) -> Option<Entity> {
        if self.tiles.get(&(cords.0, cords.1)).is_some() { return None; }

        let tile = Tile { tile_type, x: cords.0, y: cords.1 };

        let c = commands.spawn((
            tile,
            SpriteBundle {
                texture: self.texture.clone(),
                transform: Transform {
                    translation: Vec3 {
                        x: (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x + tile.x as f32 * res_grid.tile_size),
                        y: (window.resolution.height() / 2. - res_grid.tile_size / 2.) + (res_grid.offset.y + tile.y as f32 * res_grid.tile_size),
                        z: 0.0,
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

        self.tiles.insert(
            cords,
            tile,
        );

        return Some(c.id());
    }
}