
use std::default::Default;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;

use std::string::ToString;
use std::sync::Arc;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{AssetServer, AsyncReadExt};
use bevy::DefaultPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::{Assets, Camera2dBundle, Commands, Component, EventReader, Mesh, NonSend, Query, Res, ResMut, Resource, Transform, Vec2, Window, With};

use bevy::sprite::{Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::default;

use bevy::window::{WindowMode, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy_console::{ConsoleConfiguration, ConsolePlugin, PrintConsoleLine};

use bevy_file_dialog::FileDialogPlugin;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::{Color32, FontData, FontFamily, Stroke};
use bevy_inspector_egui::egui::epaint::Shadow;
use clap::builder::StyledStr;
use color_print::cformat;
use imageproc::drawing::Canvas;
use lazy_static::lazy_static;
use log::{LevelFilter};
use num::ToPrimitive;

use winit::window::Icon;

use crate::common::{BufferedLogger, Coordinates, LogMessage, PRIMARY_COLOR, Weighted};
use crate::common::io::{Load, Save};
use crate::editor_data::EditorData;
use crate::editor_data::io::EditorDataSaver;
use crate::graphics::{GraphicsResource, LegacyTextures};
use crate::graphics::tileset::legacy::LegacyTilesetLoader;
use crate::map::events::{ClearTiles, SpawnMapEntity};
use crate::map::loader::MapEntityLoader;
use crate::map::MapPlugin;
use crate::map::resources::MapEntityType;
use crate::map::systems::{set_tile_reader, spawn_sprite, tile_despawn_reader, tile_remove_reader, tile_spawn_reader, update_sprite_reader};
use crate::palettes::MeabyParam;
use crate::project::resources::Project;
use crate::region_settings::loader::RegionSettingsLoader;
use crate::tiles::components::{Offset, Tile};
use crate::tiles::TilePlugin;
use crate::ui::grid::{GridMarker, GridMaterial, GridPlugin};
use crate::ui::grid::resources::Grid;
use crate::ui::tabs::events::SpawnTab;
use crate::ui::UiPlugin;

mod tiles;
mod map;
mod ui;
mod project;
mod graphics;
mod palettes;
mod common;
mod region_settings;
mod editor_data;

lazy_static! {
    pub static ref LOGGER: BufferedLogger = BufferedLogger::new();
}

#[derive(Component)]
pub struct MouseLocationTextMarker;

#[derive(Resource)]
pub struct IsCursorCaptured(bool);

#[derive(Event)]
pub struct SwitchProject {
    pub index: u32,
}

fn main() {
    lazy_static::initialize(&LOGGER);
    log::set_logger(LOGGER.deref()).unwrap();
    log::set_max_level(LevelFilter::Info);

    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CDDA Map Editor".to_string(),
                mode: WindowMode::BorderlessFullscreen,
                ..Default::default()
            }),
            ..Default::default()
        }).disable::<LogPlugin>(), ConsolePlugin))
        .insert_resource(IsCursorCaptured(false))
        .insert_resource(ConsoleConfiguration {
            keys: vec![
                // TODO: Localize
                KeyCode::F1
            ],
            ..default()
        })
        .add_event::<LogMessage>()
        .add_systems(Startup, (
            setup,
            setup_egui,
        ))
        .add_event::<SwitchProject>()
        .add_plugins(FileDialogPlugin::new()
            .with_save_file::<Project>()
            .with_load_file::<Project>()
        )
        .add_plugins(Material2dPlugin::<GridMaterial>::default())
        .add_plugins((GridPlugin {}, MapPlugin {}, TilePlugin {}, UiPlugin {}))
        .add_systems(Update, (
            update,
            app_exit,
            switch_project,
            tile_despawn_reader,
            apply_deferred,
            tile_remove_reader,
            apply_deferred,
            set_tile_reader,
            apply_deferred,
            tile_spawn_reader,
            apply_deferred,
            spawn_sprite,
            update_sprite_reader,
        ).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<GridMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    win_windows: NonSend<WinitWindows>,
    res_grid: Res<Grid>,
    mut e_spawn_map_entity: EventWriter<SpawnMapEntity>,
    mut e_spawn_tab: EventWriter<SpawnTab>,
    mut r_images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let editor_data_saver = EditorDataSaver::new();
    let mut editor_data = editor_data_saver.load().unwrap();

    let cdda_path = "C:/CDDA/testing";

    // TODO: This is just for debug
    match &mut editor_data.config.cdda_data {
        None => {
            editor_data.config.load_cdda_dir(PathBuf::from(cdda_path));
        }
        _ => {}
    }

    let tileset_loader = LegacyTilesetLoader::new(PathBuf::from(format!(r"{}/gfx/MSX++UnDeadPeopleEdition", cdda_path)));
    let region_settings_loader = RegionSettingsLoader::new(PathBuf::from(format!(r"{}/data/json/regional_map_settings.json", cdda_path)), "default".to_string());

    let legacy_textures = LegacyTextures::new(tileset_loader, region_settings_loader, &mut r_images);
    let texture_resource = GraphicsResource::new(Box::new(legacy_textures));

    let loader = MapEntityLoader {
        path: PathBuf::from(format!(r"{}/data/json/mapgen/mall/mall_ground.json", cdda_path)),
        id: "mall_a_1".to_string(),
        cdda_data: &editor_data.config.cdda_data.as_ref().unwrap().clone(),
    };

    let map_entity = loader.load().unwrap();

    let mut project = Project::default();
    project.map_entity = map_entity;

    e_spawn_map_entity.send(SpawnMapEntity {
        map_entity: Arc::new(project.map_entity.clone())
    });

    editor_data.projects.push(project);

    for (i, project) in editor_data.projects.iter().enumerate() {
        let name = match &project.map_entity.map_type {
            MapEntityType::NestedMapgen { .. } => todo!(),
            MapEntityType::Default { om_terrain, .. } => om_terrain.clone(),
            MapEntityType::Multi { .. } => todo!(),
            MapEntityType::Nested { .. } => "Nested_TODO".to_string()
        };

        e_spawn_tab.send(SpawnTab { name, index: i as u32 });
    }

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(include_bytes!("../assets/grass.png"))
            .unwrap()
            .to_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    win_windows.windows.iter().for_each(|(_, w)| w.set_window_icon(Some(icon.clone())));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle::from(meshes.add(Cuboid::new(1., 1., 0.0).mesh())),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            material: materials.add(GridMaterial {
                tile_size: res_grid.tile_size,
                offset: Vec2::default(),
                mouse_pos: Default::default(),
                is_cursor_captured: 0,
                map_size: editor_data.get_current_project().unwrap().map_entity.size,
                scale_factor: 1.,
            }),
            ..default()
        },
        GridMarker {}
    ));

    commands.insert_resource(editor_data);
    commands.insert_resource(texture_resource);
}

fn setup_egui(
    mut contexts: EguiContexts,
) {
    let mut fonts = egui::FontDefinitions::empty();
    fonts.font_data.insert(
        "unifont".to_string(),
        FontData::from_static(include_bytes!("../assets/fonts/unifont.ttf")),
    );

    fonts.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "unifont".to_string());

    fonts.families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "unifont".to_string());

    contexts.ctx_mut().set_fonts(fonts);
    contexts.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        window_shadow: Shadow {
            extrusion: 0.0,
            color: Default::default(),
        },
        window_stroke: Stroke::NONE,
        override_text_color: Some(Color32::from_rgb(255, 255, 255)),
        window_fill: Color32::from_rgb(
            (PRIMARY_COLOR.r() * 255.).to_u8().unwrap(),
            (PRIMARY_COLOR.g() * 255.).to_u8().unwrap(),
            (PRIMARY_COLOR.b() * 255.).to_u8().unwrap(),
        ),
        ..default()
    });
}

fn update(
    res_grid: Res<Grid>,
    res_cursor: Res<IsCursorCaptured>,
    mut tiles: Query<(&mut Transform, &Coordinates, &Offset), With<Tile>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut grid_material: ResMut<Assets<GridMaterial>>,
    r_data: Res<EditorData>,
    mut e_write_line: EventWriter<PrintConsoleLine>,
) {
    for log in LOGGER.log_queue.read().unwrap().iter() {
        e_write_line.send(PrintConsoleLine::new(StyledStr::from(cformat!(r#"<g>[{}] {}</g>"#, log.level.as_str(), log.message))));
    }
    LOGGER.log_queue.write().unwrap().clear();

    let project = match r_data.get_current_project() {
        None => return,
        Some(p) => p
    };

    let grid_material = grid_material.iter_mut().next().unwrap();
    let window = q_windows.single();

    grid_material.1.offset = res_grid.offset;
    grid_material.1.tile_size = res_grid.tile_size;
    grid_material.1.mouse_pos = window.cursor_position().unwrap_or(Vec2::default());
    grid_material.1.map_size = project.map_entity.size;
    // Weird way to do this, but bevy does not let me pass a bool as a uniform for some reason
    grid_material.1.is_cursor_captured = match res_cursor.0 {
        true => 1,
        false => 0
    };
    grid_material.1.scale_factor = window.resolution.scale_factor();

    for (mut transform, coordinates, sprite_offset) in tiles.iter_mut() {
        //                                              < CENTER TO TOP LEFT >                                  < ALIGN ON GRID >
        transform.translation.x = (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x - coordinates.x as f32 * res_grid.tile_size);
        transform.translation.y = (window.resolution.height() / 2. - (res_grid.tile_size + (sprite_offset.y as f32 * (res_grid.tile_size / res_grid.default_tile_size))) / 2.) + (res_grid.offset.y - coordinates.y as f32 * res_grid.tile_size);
    }
}

fn switch_project(
    mut e_switch_project: EventReader<SwitchProject>,
    mut e_clear_tiles: EventWriter<ClearTiles>,
    mut e_spawn_map_entity: EventWriter<SpawnMapEntity>,
    mut r_editor_data: ResMut<EditorData>,
) {
    for switch_project in e_switch_project.read() {
        let new_project = r_editor_data.projects.get(switch_project.index as usize).unwrap();

        e_clear_tiles.send(ClearTiles {});

        e_spawn_map_entity.send(SpawnMapEntity {
            map_entity: Arc::new(new_project.map_entity.clone())
        });

        r_editor_data.current_project_index = switch_project.index;
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