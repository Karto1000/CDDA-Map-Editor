use std::any::Any;
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;
use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::{Asset, AssetServer};
use bevy::DefaultPlugins;
use bevy::input::keyboard::KeyboardInput;
use bevy::log::LogPlugin;
use bevy::prelude::{Assets, Bundle, Camera2dBundle, Commands, Component, EventReader, Mesh, NonSend, Query, Res, ResMut, Resource, shape, Transform, TypePath, Vec2, Vec2Swizzles, Window, With, Without};
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::default;
use bevy::window::{CursorMoved, PresentMode, WindowMode, WindowPlugin, WindowResolution};
use bevy::winit::WinitWindows;
use bevy_console::{AddConsoleCommand, ConsoleCommand, ConsoleConfiguration, ConsolePlugin, ConsoleSet, PrintConsoleLine, reply};
use bevy_console::clap::Parser;
use bevy_file_dialog::FileDialogPlugin;
use bevy_inspector_egui::{bevy_egui, egui};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiContexts};
use bevy_inspector_egui::egui::{Color32, epaint, FontData, FontFamily, Stroke};
use bevy_inspector_egui::egui::epaint::Shadow;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::builder::StyledStr;
use color_print::cformat;
use directories::ProjectDirs;
use image::Rgba;
use lazy_static::lazy_static;
use log::{Level, LevelFilter, Log, Metadata, Record};
use num::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use winit::window::Icon;

use crate::common::{BufferedLogger, Coordinates, LogMessage, MeabyWeighted, PRIMARY_COLOR, PRIMARY_COLOR_FADED, Weighted};
use crate::common::io::{Load, LoadError, Save, SaveError};
use crate::graphics::{GraphicsResource, LegacyTextures};
use crate::graphics::tileset::legacy::LegacyTilesetLoader;
use crate::grid::{GridMarker, GridMaterial, GridPlugin};
use crate::grid::resources::Grid;
use crate::map::events::{ClearTiles, SpawnMapEntity};
use crate::map::loader::MapEntityLoader;
use crate::map::MapPlugin;
use crate::map::resources::{MapEntity, MapEntityType};
use crate::map::systems::{set_tile_reader, spawn_sprite, tile_despawn_reader, tile_remove_reader, tile_spawn_reader, update_sprite_reader};
use crate::palettes::{MeabyParam, Palette};
use crate::palettes::loader::PalettesLoader;
use crate::project::resources::{Project, ProjectSaveState};
use crate::project::saver::ProjectSaver;
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

pub const CDDA_DIR: &'static str = r"C:\DEV\SelfDEV\CDDA\CDDA-Map-Editor\saves";

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
        let map: MapEntity = MapEntity::new(
            "unnamed".into(),
            Vec2::new(24., 24.),
        );

        let project = Project {
            map_entity: map,
            save_state: ProjectSaveState::NotSaved,
        };

        return Self {
            current_project_index: 0,
            projects: vec![project],
            history: vec![],
        };
    }
}

pub struct EditorDataSaver;

impl EditorDataSaver {
    pub fn new() -> Self {
        return Self {};
    }
}

impl Save<EditorData> for EditorDataSaver {
    fn save(&self, value: &EditorData) -> Result<(), SaveError> {
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(SaveError::DirectoryNotFound("".into())); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(data_dir.join("data.json"))
            .unwrap();

        let mut converted = serde_json::to_value(value).unwrap();
        let data = converted.as_object_mut().unwrap();

        let open_projects: Vec<ProjectSaveState> = value.projects.iter().map(|project| {
            match &project.save_state {
                ProjectSaveState::AutoSaved(val) => ProjectSaveState::AutoSaved(val.clone()),
                ProjectSaveState::Saved(val) => ProjectSaveState::Saved(val.clone()),
                ProjectSaveState::NotSaved => {
                    let filename = match &project.map_entity.map_type {
                        MapEntityType::NestedMapgen { .. } => todo!(),
                        MapEntityType::Default { om_terrain, .. } => om_terrain,
                        MapEntityType::Multi { .. } => todo!(),
                        MapEntityType::Nested { .. } => todo!()
                    };

                    info!("autosaving {}", filename);
                    let project_saver = ProjectSaver { directory: Box::from(data_dir) };
                    project_saver.save(project).unwrap();
                    ProjectSaveState::AutoSaved(data_dir.join(format!("auto_save_{}.map", filename)))
                }
            }
        }).collect();

        data.insert("open_projects".into(), serde_json::to_value(open_projects).unwrap());

        data.remove("projects".into());
        data.remove("current_project_index".into());

        file.write_all(serde_json::to_string(data).unwrap().as_bytes()).unwrap();

        return Ok(());
    }
}

impl Load<EditorData> for EditorDataSaver {
    fn load(&self) -> Result<EditorData, LoadError> {
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(LoadError::DirectoryNotFound); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let contents = match fs::read_to_string(data_dir.join("data.json")) {
            Err(_) => return Ok(EditorData {
                ..default()
            }),
            Ok(f) => f
        };

        let value: Map<String, Value> = serde_json::from_str(contents.as_str())
            .expect("Valid Json");

        let history_array: Vec<ProjectSaveState> = value
            .get("history")
            .expect("history Field")
            .as_array()
            .expect("Valid Array")
            .iter()
            .map(|v| serde_json::from_value::<ProjectSaveState>(v.clone()).unwrap())
            .collect();

        let projects_array: Vec<Project> = value
            .get("open_projects")
            .expect("open_projects field")
            .as_array()
            .expect("Valid array")
            .iter()
            .map(|v| {
                let state = serde_json::from_value::<ProjectSaveState>(v.clone()).unwrap();

                return match state {
                    ProjectSaveState::Saved(path) => {
                        match fs::read_to_string(path.clone()) {
                            Ok(s) => {
                                let mut project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");

                                info!("Loaded Saved Project at Path {:?}", path);

                                Some(project)
                            }
                            Err(_) => {
                                log::warn!("Could not Load Saved Project at path {:?}", path);
                                None
                            }
                        }
                    }
                    ProjectSaveState::AutoSaved(path) => {
                        match fs::read_to_string(path.clone()) {
                            Ok(s) => {
                                let project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");

                                info!("Loaded Auto saved Project at Path {:?}", path);

                                Some(project)
                            }
                            Err(_) => {
                                log::warn!("Could not Load Not Saved Project at path {:?}", path);
                                Some(Project::default())
                            }
                        }
                    }
                    ProjectSaveState::NotSaved => {
                        log::warn!("Could not open Project because it was not saved");
                        return None;
                    }
                };
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect();

        return Ok(EditorData {
            current_project_index: 0,
            projects: projects_array,
            history: history_array,
        });
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
            // update_mouse_location,
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
    mut e_log: EventWriter<LogMessage>,
    mut r_images: ResMut<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle::default());

    let tileset_loader = LegacyTilesetLoader::new(PathBuf::from(format!(r"{}/gfx/MSX++UnDeadPeopleEdition", CDDA_DIR)));
    let region_settings_loader = RegionSettingsLoader::new(PathBuf::from(format!(r"{}/data/json/regional_map_settings.json", CDDA_DIR)), "default".to_string());

    let editor_data_saver = EditorDataSaver::new();
    let legacy_textures = LegacyTextures::new(tileset_loader, region_settings_loader, &mut r_images);
    let texture_resource = GraphicsResource::new(Box::new(legacy_textures));

    let mut default_project = Project::default();
    let mut editor_data = editor_data_saver.load().unwrap();

    let loader = MapEntityLoader {
        path: PathBuf::from(format!(r"{}/data/json/mapgen/mall/mall_ground.json", CDDA_DIR)),
        id: "mall_a_1".to_string(),
    };

    let map_entity = loader.load().unwrap();

    let project: &mut Project = editor_data.get_current_project_mut().unwrap_or(&mut default_project);
    project.map_entity = map_entity;

    e_spawn_map_entity.send(SpawnMapEntity {
        map_entity: Arc::new(project.map_entity.clone())
    });

    for (i, project) in editor_data.projects.iter().enumerate() {
        let name = match &project.map_entity.map_type {
            MapEntityType::NestedMapgen { .. } => todo!(),
            MapEntityType::Default { om_terrain, .. } => om_terrain.clone(),
            MapEntityType::Multi { .. } => todo!(),
            MapEntityType::Nested { om_terrain, .. } => "Nested_TODO".to_string()
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
    // Weird way to do this but bevy does not let me pass a bool as a uniform for some reason
    grid_material.1.is_cursor_captured = match res_cursor.0 {
        true => 1,
        false => 0
    };
    grid_material.1.scale_factor = window.resolution.scale_factor() as f32;

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