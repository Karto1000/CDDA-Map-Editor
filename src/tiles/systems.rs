use bevy::input::Input;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::{EventReader, EventWriter, MouseButton, Query, Res, ResMut, Transform, Vec2Swizzles, Window, With, Without};
use bevy::window::{PrimaryWindow, WindowResized};

use crate::{EditorData, IsCursorCaptured};
use crate::common::Coordinates;
use crate::grid::GridMarker;
use crate::grid::resources::Grid;
use crate::map::events::{TileDeleteEvent, TilePlaceEvent};
use crate::tiles::components::Tile;
use crate::tiles::resources::PlaceInfo;

pub fn window_tile_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut tiles: Query<(&mut Tile, &mut Transform, &Coordinates), Without<GridMarker>>,
    res_grid: Res<Grid>,
) {
    for e in resize_reader.read() {
        for (tile, mut transform, coordinates) in tiles.iter_mut() {
            transform.translation.x = -e.width / 2. + res_grid.tile_size / 2. + res_grid.tile_size * coordinates.x as f32;
            transform.translation.y = e.height / 2. - res_grid.tile_size / 2. - res_grid.tile_size * coordinates.y as f32;
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
    mut e_set_tile: EventWriter<TilePlaceEvent>,
    mut r_place_info: ResMut<PlaceInfo>,
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    r_grid: Res<Grid>,
    r_editor_data: Res<EditorData>,
    r_captured: Res<IsCursorCaptured>,
) {
    if buttons.just_released(MouseButton::Left) {
        r_place_info.last_place_position = None
    }

    if buttons.pressed(MouseButton::Left) {
        let project = match r_editor_data.get_current_project() {
            None => return,
            Some(p) => p
        };

        let xy = match q_windows.single().cursor_position() {
            None => return,
            Some(p) => p.xy()
        };

        if r_captured.0 {
            return;
        }

        // TODO - REPLACE
        let tile_to_place: char = 'p';

        let tile_cords = Coordinates::new(
            ((xy.x + r_grid.offset.x) / r_grid.tile_size).floor() as i32,
            ((xy.y + r_grid.offset.y) / r_grid.tile_size).floor() as i32,
        );

        if tile_cords.x >= r_grid.map_size.x as i32 ||
            tile_cords.y >= r_grid.map_size.y as i32 ||
            tile_cords.x <= 0 ||
            tile_cords.y <= 0 {
            return;
        }

        if project.map_entity.tiles.get(&tile_cords).is_some() { return; }

        let tile = Tile { character: tile_to_place, fg_entity: None, bg_entity: None };
        e_set_tile.send(TilePlaceEvent { tile, coordinates: tile_cords, should_update_sprites: true });
    }
}

pub fn tile_delete_system(
    mut res_editor_data: ResMut<EditorData>,
    mut e_delete_tile: EventWriter<TileDeleteEvent>,
    buttons: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    r_grid: Res<Grid>,
    res_captured: Res<IsCursorCaptured>,
) {
    let project = match res_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    if buttons.pressed(MouseButton::Right) {
        let xy = match q_windows.single().cursor_position() {
            None => return,
            Some(p) => p.xy()
        };

        if res_captured.0 {
            return;
        }

        let tile_cords = Coordinates::new(
            ((xy.x + r_grid.offset.x) / r_grid.tile_size).floor() as i32,
            ((xy.y + r_grid.offset.y) / r_grid.tile_size).floor() as i32,
        );

        let tile = match project.map_entity.tiles.get(&tile_cords) {
            None => { return; }
            Some(t) => t
        };

        e_delete_tile.send(TileDeleteEvent {
            tile: *tile,
            coordinates: tile_cords,
        });
    }
}