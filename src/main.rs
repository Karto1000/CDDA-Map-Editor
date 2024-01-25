use std::default::Default;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, PluginGroup};
use bevy::asset::{Asset, AssetServer};
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::prelude::{Assets, Bundle, Camera2dBundle, Commands, Component, EventReader, Mesh, MouseButton, NonSend, Query, Res, ResMut, Resource, shape, Transform, TypePath, Vec2, Vec2Swizzles, Window, With, Without};
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::utils::default;
use bevy::window::{CursorMoved, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy_egui::EguiPlugin;
use winit::window::Icon;

use crate::grid::{Grid, GridMarker, GridMaterial};
use crate::grid::systems::{grid_resize_system, map_resize_system, window_grid_resize_system};
use crate::map::MapEntity;
use crate::map::system::map_save_system;
use crate::tiles::systems::{tile_delete_system, tile_place_system, tile_resize_system, window_tile_resize_system};
use crate::tiles::Tile;

mod grid;
mod tiles;
mod map;


#[derive(Resource, Debug)]
pub struct DragInfo {
    drag_started: Option<Vec2>,
    last_position: Option<Vec2>,
}

#[derive(Resource, Debug)]
pub struct PlaceInfo {
    last_place_position: Option<Vec2>,
}

#[derive(Component)]
pub struct MouseLocationTextMarker;

fn main() {
    let grid: Grid = Grid {
        tile_size: 32.0,
        map_size: Vec2 { x: 24., y: 24. },
        default_tile_size: 32.0,
        offset: Vec2::new(0., 0.),
        min_zoom: 6.,
        max_zoom: 128.,
    };

    let drag_info: DragInfo = DragInfo {
        drag_started: None,
        last_position: None,
    };

    let place_info: PlaceInfo = PlaceInfo {
        last_place_position: None
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CDDA Map Editor".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(Material2dPlugin::<GridMaterial>::default())
        .insert_resource(grid)
        .insert_resource(drag_info)
        .insert_resource(place_info)
        .add_systems(Startup, (setup))
        .add_systems(Update, (
            update,
            window_grid_resize_system,
            drag_system,
            window_tile_resize_system,
            tile_place_system,
            update_mouse_location,
            grid_resize_system,
            tile_resize_system,
            map_save_system,
            tile_delete_system,
            map_resize_system
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<GridMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query_windows: Query<&Window, With<PrimaryWindow>>,
    win_windows: NonSend<WinitWindows>,
    res_grid: Res<Grid>,
) {
    commands.spawn(Camera2dBundle::default());
    let window = query_windows.single();

    let grass = asset_server.load("grass.png");

    let map: MapEntity = MapEntity::new(
        "test_tile_01".into(),
        res_grid.map_size,
        grass,
    );

    let window_width = window.physical_width();
    let window_height = window.physical_height();

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("./assets/grass.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    win_windows.windows.iter().for_each(|(_, w)| w.set_window_icon(Some(icon.clone())));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Box::new(1., 1., 0.0).into()).into(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            material: materials.add(GridMaterial {
                viewport_width: window_width as f32,
                viewport_height: window_height as f32,
                tile_size: res_grid.tile_size,
                offset: Vec2::default(),
                mouse_pos: Default::default(),
                map_size: res_grid.map_size,
            }),
            ..default()
        },
        GridMarker {}
    ));

    commands.spawn((
        TextBundle::from_section(
            "0, 0",
            TextStyle {
                font: asset_server.load("fonts/unifont.ttf"),
                font_size: 24.0,
                ..default()
            },
        )
            .with_text_alignment(TextAlignment::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            }),
        MouseLocationTextMarker {}
    ));

    commands.insert_resource(map);
}


fn update(
    res_grid: Res<Grid>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut grid_material: ResMut<Assets<GridMaterial>>,
) {
    let grid_material = grid_material.iter_mut().next().unwrap();
    let window = q_windows.single();

    grid_material.1.offset = res_grid.offset;
    grid_material.1.tile_size = res_grid.tile_size;
    grid_material.1.mouse_pos = window.cursor_position().unwrap_or(Vec2::default());
    grid_material.1.map_size = res_grid.map_size;

    for (tile, mut transform) in tiles.iter_mut() {
        //                                              < CENTER TO TOP LEFT >                                  < ALIGN ON GRID >
        transform.translation.x = (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x - tile.x as f32 * res_grid.tile_size);
        transform.translation.y = (window.resolution.height() / 2. - res_grid.tile_size / 2.) + (res_grid.offset.y - tile.y as f32 * res_grid.tile_size);
    }
}


fn drag_system(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    mut cursor_motion: EventReader<CursorMoved>,
    mut res_grid: ResMut<Grid>,
    mut res_drag: ResMut<DragInfo>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if buttons.just_pressed(MouseButton::Middle) {
        let xy = q_windows.single().cursor_position().unwrap_or(Vec2::default()).xy();
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

fn update_mouse_location(
    mut event_cursor: EventReader<CursorMoved>,
    mut location_text: Query<(&mut Text, &MouseLocationTextMarker)>,
    query_windows: Query<&Window, With<PrimaryWindow>>,
    res_grid: Res<Grid>,
) {
    let mut text = location_text.single_mut();
    let window = query_windows.single();
    let xy = window.cursor_position().unwrap_or(Vec2::default()).xy();

    for _ in event_cursor.read() {
        let pos = ((xy + res_grid.offset) / res_grid.tile_size).floor();
        text.0.sections[0].value = format!("{}, {}", pos.x, pos.y);
    }
}

