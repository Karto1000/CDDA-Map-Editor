use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::Vec2;
use bevy::prelude::{Commands, CursorMoved, EventReader, MouseButton, Query, Res, ResMut, Transform, Vec2Swizzles, Window, With, Without};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::tiles::components::Tile;
use crate::ui::grid::{DragInfo, Grid, GridMarker};

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
                let window = q_windows.single();

                let old_size = r_grid.tile_size.clone();
                r_grid.tile_size = (old_size + event.y * 2.).clamp(r_grid.min_zoom, r_grid.max_zoom);
                let new_size = r_grid.tile_size.clone();

                let old_position = (window.cursor_position().unwrap_or(Vec2::new(0., 0.)) + r_grid.offset) / old_size;
                let new_position = (window.cursor_position().unwrap_or(Vec2::new(0., 0.)) + r_grid.offset) / new_size;
                r_grid.offset -= (new_position - old_position) * new_size;

                for (_, mut transform) in tiles.iter_mut() {
                    transform.scale.x = r_grid.tile_size / r_grid.default_tile_size;
                    transform.scale.y = r_grid.tile_size / r_grid.default_tile_size;
                }
            }
            MouseScrollUnit::Pixel => todo!()
        }
    }
}

pub fn drag_system(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
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