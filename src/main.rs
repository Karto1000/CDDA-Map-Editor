use std::default::Default;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, PluginGroup};
use bevy::asset::{Asset, AssetServer};
use bevy::DefaultPlugins;
use bevy::input::Input;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::{Assets, Bundle, Camera2dBundle, Commands, Component, EventReader, Mesh, MouseButton, NonSend, Query, Res, ResMut, Resource, shape, Transform, TypePath, Vec2, Vec2Swizzles, Vec3, Window, With, Without};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, SpriteBundle};
use bevy::utils::default;
use bevy::window::{CursorMoved, WindowPlugin, WindowResized};
use bevy::winit::WinitWindows;
use bevy_egui::EguiPlugin;
use winit::window::Icon;

use crate::mapgen::map_entity::{MapEntity, Tile, TileType};

mod mapgen;

#[derive(Component)]
pub struct GridMarker;

#[derive(Resource)]
pub struct Grid {
    tile_size: f32,
    default_tile_size: f32,
    offset: Vec2,

    min_zoom: f32,
    max_zoom: f32,
}

#[derive(Resource, Debug)]
pub struct DragInfo {
    drag_started: Option<Vec2>,
    last_position: Option<Vec2>,
}

fn main() {
    let grid: Grid = Grid {
        tile_size: 32.0,
        default_tile_size: 32.0,
        offset: Vec2::new(0., 0.),
        min_zoom: 6.,
        max_zoom: 128.,
    };

    let drag_info: DragInfo = DragInfo {
        drag_started: None,
        last_position: None,
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
        .add_systems(Startup, (setup, set_window_icon))
        .add_systems(Update, (update, window_resize_system, drag_system, grid_resize_system))
        .run();
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

    for (tile, mut transform) in tiles.iter_mut() {
        //                                              < CENTER TO TOP LEFT >                                  < ALIGN ON GRID >
        transform.translation.x = (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x + tile.x as f32 * res_grid.tile_size);
        transform.translation.y = (window.resolution.height() / 2. - res_grid.tile_size / 2.) + (res_grid.offset.y + tile.y as f32 * res_grid.tile_size);
    }
}

fn window_resize_system(
    mut resize_reader: EventReader<WindowResized>,
    mut grid: Query<&mut Transform, With<GridMarker>>,
    mut grid_material: ResMut<Assets<GridMaterial>>,
    mut tiles: Query<(&mut Tile, &mut Transform), Without<GridMarker>>,
    res_grid: Res<Grid>,
) {
    for e in resize_reader.read() {
        let mut grid = grid.iter_mut().next().unwrap();
        let grid_material = grid_material.iter_mut().next().unwrap();

        grid.scale.x = e.width;
        grid.scale.y = e.height;

        grid_material.1.viewport_width = e.width / 2.;
        grid_material.1.viewport_height = e.height;

        for (tile, mut transform) in tiles.iter_mut() {
            transform.translation.x = -e.width / 2. + res_grid.tile_size / 2. + res_grid.tile_size * tile.x as f32;
            transform.translation.y = e.height / 2. - res_grid.tile_size / 2. - res_grid.tile_size * tile.y as f32;
        };
    }
}

fn grid_resize_system(
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

fn drag_system(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    mut cursor_motion: EventReader<CursorMoved>,
    mut res_grid: ResMut<Grid>,
    mut res_drag: ResMut<DragInfo>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        let xy = q_windows.single().cursor_position().unwrap().xy();
        commands.insert_resource(DragInfo {
            drag_started: Some(xy),
            last_position: Some(xy),
        })
    }

    if buttons.just_released(MouseButton::Right) {
        res_drag.last_position = None;
        res_drag.drag_started = None
    }

    if buttons.pressed(MouseButton::Right) {
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

fn set_window_icon(windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("./assets/grass.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<GridMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    res_grid: Res<Grid>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut map: MapEntity = MapEntity::new("NewHouse".into());

    let window_width = window_query.single().physical_width();
    let window_height = window_query.single().physical_height();

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
            }),
            ..default()
        },
        GridMarker {}
    ));

    map.set_tile_at(
        (0, 0),
        TileType::Test,
    );
    map.set_tile_at(
        (1, 1),
        TileType::Test,
    );
    map.set_tile_at(
        (1, 0),
        TileType::Test,
    );
    map.set_tile_at(
        (0, 2),
        TileType::Test,
    );

    let grass = asset_server.load("grass.png");

    map.tiles.iter().for_each(|(k, v)| {
        commands.spawn((
            *v,
            SpriteBundle {
                texture: grass.clone(),
                transform: Transform {
                    translation: Vec3 {
                        x: k.0 as f32 * res_grid.tile_size,
                        y: k.1 as f32 * res_grid.tile_size,
                        z: 0.0,
                    },
                    ..default()
                },
                ..default()
            },
        ));
    });

    commands.insert_resource(map);
}


#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GridMaterial {
    #[uniform(0)]
    viewport_width: f32,
    #[uniform(1)]
    viewport_height: f32,
    #[uniform(2)]
    tile_size: f32,
    #[uniform(3)]
    offset: Vec2,
    #[uniform(4)]
    mouse_pos: Vec2
}


impl Material2d for GridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid.wgsl".into()
    }
}