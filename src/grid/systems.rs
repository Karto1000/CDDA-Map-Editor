use bevy::asset::Assets;
use bevy::input::Input;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::Vec2;
use bevy::prelude::{Commands, CursorMoved, EventReader, KeyCode, MouseButton, Query, Res, ResMut, Transform, Vec2Swizzles, Window, With, Without};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::EditorData;
use crate::grid::{DragInfo, Grid, GridMarker, GridMaterial};
use crate::tiles::Tile;

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
    mut res_grid: ResMut<Grid>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    for event in scroll_event.read() {
        match event.unit {
            MouseScrollUnit::Line => {
                if res_grid.tile_size <= res_grid.min_zoom && event.y <= -1. { return; }
                if res_grid.tile_size >= res_grid.max_zoom && event.y >= 1. { return; }

                let window = q_windows.single();

                let original_tile_amount = Vec2::new(
                    window.resolution.width() / res_grid.tile_size,
                    window.resolution.height() / res_grid.tile_size
                );

                let size = res_grid.tile_size.clone();
                res_grid.tile_size = size + event.y * 2.;

                let new_tile_amount = Vec2::new(
                    window.resolution.width() / res_grid.tile_size,
                    window.resolution.height() / res_grid.tile_size
                );

                let pixels_shifted = original_tile_amount - new_tile_amount;
                let offset = res_grid.offset.clone();

                res_grid.offset += (((window.cursor_position().unwrap() - pixels_shifted) + offset) / size) * event.y;

                for (_, mut transform) in tiles.iter_mut() {
                    transform.scale.x = res_grid.tile_size / res_grid.default_tile_size;
                    transform.scale.y = res_grid.tile_size / res_grid.default_tile_size;
                }
            }
            MouseScrollUnit::Pixel => panic!("Not Implemented")
        }
    }
}

pub fn map_resize_system(
    mut res_grid: ResMut<Grid>,
    mut res_editor_data: ResMut<EditorData>,
    keys: Res<Input<KeyCode>>,
) {
    let project = match res_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    if keys.pressed(KeyCode::Right) {
        res_grid.map_size.x += 1.;
    }

    if keys.pressed(KeyCode::Down) {
        res_grid.map_size.y += 1.;
    }

    if keys.pressed(KeyCode::Left) {
        res_grid.map_size.x -= 1.;
    }

    if keys.pressed(KeyCode::Up) {
        res_grid.map_size.y -= 1.;
    }

    project.map_entity.size = res_grid.map_size
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