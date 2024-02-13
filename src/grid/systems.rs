use bevy::input::Input;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::Vec2;
use bevy::prelude::{Commands, CursorMoved, EventReader, KeyCode, MouseButton, Query, Res, ResMut, Transform, Vec2Swizzles, Window, With, Without};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::EditorData;
use crate::grid::{DragInfo, Grid, GridMarker};
use crate::tiles::components::Tile;

pub fn window_grid_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut grid: Query<&mut Transform, With<GridMarker>>,
) {
    for e in resize_reader.read() {
        let mut grid = grid.iter_mut().next().unwrap();

        grid.scale.x = e.width;
        grid.scale.y = e.height;
    }
}

pub fn grid_resize_system(
    mut scroll_event: EventReader<MouseWheel>,
    mut r_grid: ResMut<Grid>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    for event in scroll_event.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                if r_grid.tile_size <= r_grid.min_zoom && event.y <= -1. { return; }
                if r_grid.tile_size >= r_grid.max_zoom && event.y >= 1. { return; }

                let window = q_windows.single();

                let original_tile_amount = Vec2::new(
                    window.resolution.width() / r_grid.tile_size,
                    window.resolution.height() / r_grid.tile_size,
                );

                let size = r_grid.tile_size.clone();
                r_grid.tile_size = size + event.y * 2.;

                let new_tile_amount = Vec2::new(
                    window.resolution.width() / r_grid.tile_size,
                    window.resolution.height() / r_grid.tile_size,
                );

                let pixels_shifted = original_tile_amount - new_tile_amount;
                let offset = r_grid.offset.clone();

                r_grid.offset += (((window.cursor_position().unwrap() - pixels_shifted) + offset) / size) * event.y;

                for (_, mut transform) in tiles.iter_mut() {
                    transform.scale.x = r_grid.tile_size / r_grid.default_tile_size;
                    transform.scale.y = r_grid.tile_size / r_grid.default_tile_size;
                }
            }
            MouseScrollUnit::Pixel => panic!("Not Implemented")
        }
    }
}

pub fn drag_system(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    mut cursor_motion: EventReader<CursorMoved>,
    mut res_grid: ResMut<Grid>,
    mut res_drag: ResMut<DragInfo>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = q_windows.single();

    if buttons.just_pressed(MouseButton::Middle) {
        let xy = window.cursor_position().unwrap_or(Vec2::default()).xy();
        commands.insert_resource(DragInfo {
            drag_started: Some(xy),
            last_position: Some(xy),
        })
    }

    if buttons.just_released(MouseButton::Middle) {
        res_drag.last_position = None;
        res_drag.drag_started = None
    }

    if buttons.pressed(MouseButton::Middle) {
        match cursor_motion.read().last() {
            None => return,
            Some(m) => {
                let offset = res_grid.offset.clone();
                res_grid.offset = offset + res_drag.last_position.unwrap_or(m.position) - m.position;
                res_drag.last_position = Some(m.position);
            }
        }
    }
}