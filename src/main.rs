use std::any::Any;
use std::collections::HashMap;
use std::default::Default;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{Asset, AssetServer};
use bevy::DefaultPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::{Assets, Bundle, Camera2dBundle, Commands, Component, EventReader, Mesh, NonSend, Query, Res, ResMut, Resource, Transform, TypePath, Vec2, Vec2Swizzles, Window, With};
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::default;
use bevy::window::{WindowMode, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy_console::{AddConsoleCommand, ConsoleConfiguration, ConsolePlugin, PrintConsoleLine};
use bevy_console::clap::Parser;
use bevy_file_dialog::FileDialogPlugin;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::{Color32, FontData, FontFamily, Stroke};
use bevy_inspector_egui::egui::epaint::Shadow;
use clap::builder::StyledStr;
use color_print::cformat;
use lazy_static::lazy_static;
use log::{LevelFilter, Log};
use num::ToPrimitive;
use serde::{Deserialize, Serialize};
use winit::window::Icon;

use crate::common::{BufferedLogger, Coordinates, LogMessage, PRIMARY_COLOR};
use crate::common::io::{Load, Save};
use crate::graphics::{GraphicsResource, LegacyTextures};
use crate::graphics::tileset::legacy::LegacyTilesetLoader;
use crate::grid::{GridMarker, GridMaterial, GridPlugin};
use crate::grid::resources::Grid;
use crate::map::events::{ClearTiles, SpawnMapEntity};
use crate::map::loader::MapEntityLoader;
use crate::map::MapPlugin;
use crate::map::resources::{MapEntity, Single};
use crate::map::systems::{set_tile_reader, spawn_sprite, tile_despawn_reader, tile_remove_reader, tile_spawn_reader, update_sprite_reader};
use crate::palettes::loader::PalettesLoader;
use crate::palettes::Palette;
use crate::project::resources::{Project, ProjectSaveState};
use crate::region_settings::loader::RegionSettingsLoader;
use crate::tiles::components::{Offset, Tile};
use crate::tiles::TilePlugin;
use crate::ui::tabs::events::SpawnTab;
use crate::ui::UiPlugin;

mod grid;
mod tiles;
mod map;
mod ui;
mod project;
mod graphics;
mod palettes;
mod common;
mod region_settings;

pub const CDDA_DIR: &'static str = r"C:\CDDA\testing";

lazy_static! {
    pub static ref ALL_PALETTES: HashMap<String, Palette> = PalettesLoader::new(PathBuf::from(format!(r"{}/data/json/mapgen_palettes", CDDA_DIR))).load().unwrap();
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

#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct EditorData {
    pub current_project_index: u32,
    pub projects: Vec<Project>,
    pub history: Vec<ProjectSaveState>,
}

impl EditorData {
    pub fn get_current_project(&self) -> Option<&Project> {
        return self.projects.get(self.current_project_index as usize);
    }

    pub fn get_current_project_mut(&mut self) -> Option<&mut Project> {
        return self.projects.get_mut(self.current_project_index as usize);
    }
}

impl Default for EditorData {
    fn default() -> Self {
        let map: Single = Single {
            om_terrain: "unnamed".to_string(),
            tile_selection: Default::default(),
            tiles: Default::default(),
        };

        let project = Project {
            map_entity: MapEntity::Single(map),
            save_state: ProjectSaveState::NotSaved,
        };

        return Self {
            current_project_index: 0,
            projects: vec![project],
            history: vec![],
        };
    }
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
        .add_systems(Startup, (setup, setup_egui))
        .add_event::<SwitchProject>()
        .add_plugins(FileDialogPlugin::new()
            .with_save_file::<Project>()
            .with_load_file::<Project>()
        )
        .add_plugins(Material2dPlugin::<GridMaterial>::default())
        .add_plugins((GridPlugin {}, MapPlugin {}, TilePlugin {}, UiPlugin {}))
        .add_systems(Update, (
            update,
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
    mut e_log: EventWriter<LogMessage>,
    mut r_images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let tileset_loader = LegacyTilesetLoader::new(PathBuf::from(format!(r"{}/gfx/MSX++UnDeadPeopleEdition", CDDA_DIR)));
    let region_settings_loader = RegionSettingsLoader::new(PathBuf::from(format!(r"{}/data/json/regional_map_settings.json", CDDA_DIR)), "default".to_string());

    let legacy_textures = LegacyTextures::new(tileset_loader, region_settings_loader, &mut r_images);
    let texture_resource = GraphicsResource::new(Box::new(legacy_textures));

    let mut default_project = Project::default();

    let loader = MapEntityLoader {
        path: PathBuf::from(format!(r"{}/data/json/mapgen/house/house01.json", CDDA_DIR)),
        id: "house_01".to_string(),
    };

    let mut editor_data = EditorData::default();
    let map_entity = MapEntity::Single(loader.load().unwrap());

    let project: &mut Project = editor_data.get_current_project_mut().unwrap_or(&mut default_project);
    project.map_entity = map_entity;

    e_spawn_map_entity.send(SpawnMapEntity {
        map_entity: Arc::new(project.map_entity.clone())
    });

    for (i, project) in editor_data.projects.iter().enumerate() {
        let name = match &project.map_entity {
            MapEntity::Single(s) => s.om_terrain.clone(),
            _ => todo!()
        };

        e_spawn_tab.send(SpawnTab { name, index: i as u32 });
    }

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
            mesh: Mesh2dHandle::from(meshes.add(Cuboid::new(1., 1., 0.0).mesh())),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            material: materials.add(GridMaterial {
                tile_size: res_grid.tile_size,
                offset: Vec2::default(),
                mouse_pos: Default::default(),
                is_cursor_captured: 0,
                map_size: editor_data.get_current_project().unwrap().map_entity.size(),
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
    grid_material.1.map_size = project.map_entity.size();
    // Weird way to do this but bevy does not let me pass a bool as a uniform for some reason
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