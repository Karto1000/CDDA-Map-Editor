use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

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
use bevy_file_dialog::FileDialogPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use winit::window::Icon;

use crate::common::{Coordinates, MeabyWeighted, Weighted};
use crate::common::io::{Load, LoadError, Save, SaveError};
use crate::graphics::{GraphicsResource, LegacyTextures};
use crate::graphics::tileset::legacy::LegacyTilesetLoader;
use crate::grid::{GridMarker, GridMaterial, GridPlugin};
use crate::grid::resources::Grid;
use crate::map::events::{ClearTiles, SpawnMapEntity};
use crate::map::loader::MapEntityImporter;
use crate::map::MapPlugin;
use crate::map::resources::MapEntity;
use crate::map::systems::{set_tile_reader, spawn_sprite, tile_despawn_reader, tile_remove_reader, tile_spawn_reader, update_sprite_reader};
use crate::palettes::{Identifier, MapObjectId, Palette};
use crate::palettes::loader::PalettesLoader;
use crate::project::resources::{Project, ProjectSaveState};
use crate::project::saver::ProjectSaver;
use crate::region_settings::loader::RegionSettingsLoader;
use crate::tiles::components::Tile;
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

    #[serde(skip)]
    pub all_palettes: HashMap<String, Palette>,
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
            all_palettes: HashMap::new(),
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
                    info!("autosaving {}", project.map_entity.map_type.get_name().clone());
                    let project_saver = ProjectSaver { directory: Box::from(data_dir) };
                    project_saver.save(project).unwrap();
                    ProjectSaveState::AutoSaved(data_dir.join(format!("auto_save_{}.map", project.map_entity.map_type.get_name().clone())))
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
        let palettes_loader = PalettesLoader::new(PathBuf::from(r"C:\CDDA\testing\data\json\mapgen_palettes"));
        let palettes = palettes_loader.load().unwrap();

        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(LoadError::DirectoryNotFound); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let contents = match fs::read_to_string(data_dir.join("data.json")) {
            Err(_) => return Ok(EditorData {
                all_palettes: palettes,
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

                                let project_specific_palettes: Vec<&Palette> = project.map_entity.palettes.iter().map(|id| {
                                    palettes.get(&id.id).unwrap()
                                }).collect();

                                project.map_entity.palettes.clear();

                                for palette in project_specific_palettes {
                                    project.map_entity.add_palette(&palettes, palette);
                                }

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
                                let mut project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");

                                let project_specific_palettes: Vec<&Palette> = project.map_entity.palettes.iter().map(|id| {
                                    palettes.get(&id.id).unwrap()
                                }).collect();

                                project.map_entity.palettes.clear();

                                for palette in project_specific_palettes {
                                    project.map_entity.add_palette(&palettes, palette);
                                }

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
            all_palettes: palettes,
        });
    }
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
        .add_event::<SwitchProject>()
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(FileDialogPlugin::new()
            .with_save_file::<Project>()
            .with_load_file::<Project>()
        )
        .add_plugins(Material2dPlugin::<GridMaterial>::default())
        .add_plugins((GridPlugin {}, MapPlugin {}, TilePlugin {}, UiPlugin {}))
        .add_systems(Update, (
            update,
            update_mouse_location,
            app_exit,
            switch_project,
            tile_despawn_reader,
            apply_deferred,
            tile_remove_reader,
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

    let tileset_loader = LegacyTilesetLoader::new(PathBuf::from(r"C:\CDDA\testing\gfx\MSX++UnDeadPeopleEdition"));
    let region_settings_loader = RegionSettingsLoader::new(PathBuf::from(r"C:\CDDA\testing\data\json\regional_map_settings.json"), "default".to_string());

    let editor_data_saver = EditorDataSaver::new();
    let legacy_textures = LegacyTextures::new(tileset_loader, region_settings_loader, &mut r_images);
    let texture_resource = GraphicsResource::new(Box::new(legacy_textures));

    let mut default_project = Project::default();
    let mut editor_data = editor_data_saver.load().unwrap();

    let loader = MapEntityImporter::new(
        PathBuf::from(r"C:\CDDA\testing\data\json\mapgen\house\house01.json"),
        "house_01".to_string(),
        &editor_data.all_palettes,
    );

    let map_entity = loader.load().unwrap();

    let project: &mut Project = editor_data.get_current_project_mut().unwrap_or(&mut default_project);
    project.map_entity = map_entity;

    // project.map_entity.terrain.insert('g', MapObjectId::from("t_grass"));
    // project.map_entity.furniture.insert('g', MapObjectId::from("f_chair"));
    // project.map_entity.add_palette(&palette);

    e_spawn_map_entity.send(SpawnMapEntity {
        map_entity: Arc::new(project.map_entity.clone())
    });

    for (i, project) in editor_data.projects.iter().enumerate() {
        e_spawn_tab.send(SpawnTab { name: project.map_entity.map_type.get_name().clone(), index: i as u32 });
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
            mesh: meshes.add(shape::Box::new(1., 1., 0.0).into()).into(),
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
    mut tiles: Query<(&mut Tile, &mut Transform, &Coordinates), Without<GridMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut grid_material: ResMut<Assets<GridMaterial>>,
    r_data: Res<EditorData>,
) {
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

    for (tile, mut transform, coordinates) in tiles.iter_mut() {
        //                                              < CENTER TO TOP LEFT >                                  < ALIGN ON GRID >
        transform.translation.x = (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x - coordinates.x as f32 * res_grid.tile_size);
        transform.translation.y = (window.resolution.height() / 2. - res_grid.tile_size / 2.) + (res_grid.offset.y - coordinates.y as f32 * res_grid.tile_size);
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