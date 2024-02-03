use std::collections::HashMap;
use std::default::Default;
use std::ops::Deref;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{Asset, AssetServer};
use bevy::DefaultPlugins;
use bevy::prelude::{Assets, Bundle, Camera2dBundle, Commands, Component, EventReader, Mesh, NonSend, Query, Res, ResMut, Resource, shape, Transform, TypePath, Vec2, Vec2Swizzles, Window, With, Without};
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::utils::default;
use bevy::window::{CursorMoved, WindowMode, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy_egui::EguiPlugin;
use bevy_file_dialog::FileDialogPlugin;
use winit::window::Icon;

use crate::grid::{GridMarker, GridMaterial, GridPlugin};
use crate::grid::resources::Grid;
use crate::hotbar::HotbarPlugin;
use crate::hotbar::tabs::SpawnTab;
use crate::map::{MapPlugin, TilePlaceEvent};
use crate::map::resources::MapEntity;
use crate::project::{EditorData, EditorDataSaver, Project};
use crate::project::loader::Load;
use crate::project::saver::Save;
use crate::tiles::{Tile, TilePlugin, TileType};

mod grid;
mod tiles;
mod map;
mod hotbar;
mod project;


#[derive(Component)]
pub struct MouseLocationTextMarker;

#[derive(Resource)]
pub struct IsCursorCaptured(bool);

#[derive(Resource)]
pub struct TextureResource {
    pub textures: HashMap<TileType, Handle<Image>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CDDA Map Editor".to_string(),
                mode: WindowMode::BorderlessFullscreen,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(IsCursorCaptured(false))
        .add_systems(Startup, setup)
        .add_plugins(EguiPlugin)
        .add_plugins(FileDialogPlugin::new()
            .with_save_file::<MapEntity>()
            .with_load_file::<MapEntity>()
        )
        .add_plugins(Material2dPlugin::<GridMaterial>::default())
        .add_plugins((GridPlugin {}, MapPlugin {}, TilePlugin {}, HotbarPlugin {}))
        .add_systems(Update, (update, update_mouse_location, map_loaded, app_exit))
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
    mut e_set_tile: EventWriter<TilePlaceEvent>,
    mut e_spawn_tab: EventWriter<SpawnTab>,
) {
    commands.spawn(Camera2dBundle::default());
    let window = query_windows.single();

    let grass = asset_server.load("grass.png");

    let mut textures: HashMap<TileType, Handle<Image>> = HashMap::new();
    textures.insert(TileType::Test, grass);

    let mut editor_data = EditorDataSaver {}.load().unwrap();
    editor_data.get_current_project_mut().unwrap_or(&mut Project::default()).map_entity.spawn(&mut e_set_tile);

    println!("{:?}", editor_data.projects);
    for project in editor_data.projects.iter() {
        e_spawn_tab.send(SpawnTab { project: (*project).clone() });
    }

    let texture_resource = TextureResource { textures };

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
                is_cursor_captured: 0,
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

    commands.insert_resource(editor_data);
    commands.insert_resource(texture_resource);
}


fn update(
    res_grid: Res<Grid>,
    res_cursor: Res<IsCursorCaptured>,
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
    // Weird way to do this but bevy does not let me pass a bool as a uniform for some reason
    grid_material.1.is_cursor_captured = match res_cursor.0 {
        true => 1,
        false => 0
    };

    for (tile, mut transform) in tiles.iter_mut() {
        //                                              < CENTER TO TOP LEFT >                                  < ALIGN ON GRID >
        transform.translation.x = (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x - tile.x as f32 * res_grid.tile_size);
        transform.translation.y = (window.resolution.height() / 2. - res_grid.tile_size / 2.) + (res_grid.offset.y - tile.y as f32 * res_grid.tile_size);
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

fn map_loaded(
    mut ev_loaded: EventReader<bevy_file_dialog::DialogFileLoaded<MapEntity>>,
    mut res_editor_data: ResMut<EditorData>,
    mut e_set_tile: EventWriter<TilePlaceEvent>,
) {
    let project = match res_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    for ev in ev_loaded.read() {
        let map_entity: MapEntity = serde_json::from_slice(ev.contents.as_slice()).unwrap();

        project.map_entity.load(
            &mut e_set_tile,
            &map_entity,
        );
    }
}

fn app_exit(
    mut e_exit: EventReader<AppExit>,
    res_editor_data: Res<EditorData>,
) {
    for _ in e_exit.read() {
        let save_data_saver = EditorDataSaver {};
        save_data_saver.save(&res_editor_data).unwrap();
    }
}