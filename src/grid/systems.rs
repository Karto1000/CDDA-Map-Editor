use bevy::asset::Assets;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::{EventReader, Query, ResMut, Transform, With, Without};
use bevy::window::WindowResized;

use crate::grid::{Grid, GridMarker, GridMaterial};
use crate::tiles::Tile;

pub fn window_grid_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut grid: Query<&mut Transform, With<GridMarker>>,
    mut grid_material: ResMut<Assets<GridMaterial>>,
) {
    for e in resize_reader.read() {
        let mut grid = grid.iter_mut().next().unwrap();
        let grid_material = grid_material.iter_mut().next().unwrap();

        grid.scale.x = e.width;
        grid.scale.y = e.height;

        grid_material.1.viewport_width = e.width / 2.;
        grid_material.1.viewport_height = e.height;
    }
}

pub fn grid_resize_system(
    mut scroll_event: EventReader<MouseWheel>,
    mut res_grid: ResMut<Grid>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
) {
    for event in scroll_event.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                if res_grid.tile_size <= res_grid.min_zoom && event.y <= -1. { return; }
                if res_grid.tile_size >= res_grid.max_zoom && event.y >= 1. { return; }

                let size = res_grid.tile_size.clone();
                res_grid.tile_size = size + event.y * 2.;

                for (_, mut transform) in tiles.iter_mut() {
                    transform.scale.x = res_grid.tile_size / res_grid.default_tile_size;
                    transform.scale.y = res_grid.tile_size / res_grid.default_tile_size;
                }
            }
            MouseScrollUnit::Pixel => panic!("Not Implemented")
        }
    }
}