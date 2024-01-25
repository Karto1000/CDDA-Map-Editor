use bevy::input::Input;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::{Commands, EventReader, MouseButton, Query, Res, ResMut, Transform, Vec2, Vec2Swizzles, Window, With, Without};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::grid::{Grid, GridMarker};
use crate::mapgen::map_entity::MapEntity;
use crate::PlaceInfo;
use crate::tiles::{Tile, TileType};

pub fn window_tile_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
    res_grid: Res<Grid>,
) {
    for e in resize_reader.read() {
        for (tile, mut transform) in tiles.iter_mut() {
            transform.translation.x = -e.width / 2. + res_grid.tile_size / 2. + res_grid.tile_size * tile.x as f32;
            transform.translation.y = e.height / 2. - res_grid.tile_size / 2. - res_grid.tile_size * tile.y as f32;
        };
    }
}

pub fn tile_resize_system(
    mut scroll_event: EventReader<MouseWheel>,
    res_grid: ResMut<Grid>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
) {
    for event in scroll_event.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                for (_, mut transform) in tiles.iter_mut() {
                    transform.scale.x = res_grid.tile_size / res_grid.default_tile_size;
                    transform.scale.y = res_grid.tile_size / res_grid.default_tile_size;
                }
            }
            MouseScrollUnit::Pixel => panic!("Not Implemented")
        }
    }
}

pub fn tile_place_system(
    mut commands: Commands,
    mut res_map: ResMut<MapEntity>,
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    res_grid: Res<Grid>,
    mut res_place_info: ResMut<PlaceInfo>,
) {
    if buttons.just_released(MouseButton::Left) {
        res_place_info.last_place_position = None
    }

    if buttons.pressed(MouseButton::Left) {
        let xy = match q_windows.single().cursor_position() {
            None => return,
            Some(p) => p.xy()
        };

        // TODO - REPLACE
        let tile_to_place: TileType = TileType::Test;

        let tile_cords = Vec2::new(
            ((xy.x + res_grid.offset.x) / res_grid.tile_size).floor(),
            ((xy.y + res_grid.offset.y) / res_grid.tile_size).floor(),
        );

        res_map.set_tile_at(
            &mut commands,
            (-tile_cords.x as i32, -tile_cords.y as i32),
            tile_to_place,
            &res_grid,
            q_windows.single(),
        );

        let dist = (xy + res_grid.offset) - (res_place_info.last_place_position.unwrap_or(xy) + res_grid.offset);
        let grid_dist = (dist / res_grid.tile_size).round().abs();
        let dir = dist.clamp(Vec2::new(-1., -1.), Vec2::new(1., 1.));

        res_place_info.last_place_position = Some(xy);

        match grid_dist.y.abs() > grid_dist.x.abs() {
            true => {
                // Y in greater
                let slope = grid_dist.x / grid_dist.y;

                for y in 0..grid_dist.y as i32 {
                    let tile_cords = Vec2::new(
                        ((xy.x + res_grid.offset.x) / res_grid.tile_size + slope * dir.x).floor(),
                        ((xy.y + res_grid.offset.y) / res_grid.tile_size + y as f32 * dir.y).floor(),
                    );

                    res_map.set_tile_at(
                        &mut commands,
                        (-tile_cords.x as i32, -tile_cords.y as i32),
                        tile_to_place,
                        &res_grid,
                        q_windows.single(),
                    );
                };
            }
            false => {
                // X in greater
                let slope = grid_dist.y / grid_dist.x;

                for x in 0..grid_dist.x as i32 {
                    let tile_cords = Vec2::new(
                        ((xy.x + res_grid.offset.x) / res_grid.tile_size + x as f32 * dir.x).floor(),
                        ((xy.y + res_grid.offset.y) / res_grid.tile_size + slope * dir.y).floor(),
                    );

                    res_map.set_tile_at(
                        &mut commands,
                        (-tile_cords.x as i32, -tile_cords.y as i32),
                        tile_to_place,
                        &res_grid,
                        q_windows.single(),
                    );
                };
            }
        };
    }
}