use std::default::Default;
use std::io::Write;
use std::ops::Deref;
use std::string::ToString;
use std::sync::Arc;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::AsyncReadExt;
use bevy::DefaultPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::{Assets, Camera2dBundle, Commands, Component, EventReader, NonSend, Query, Res, ResMut, Resource, Transform, Vec2, Window, With};
use bevy::sprite::Material2dPlugin;
use bevy::utils::default;
use bevy::window::{WindowMode, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy_console::{ConsoleConfiguration, ConsolePlugin, PrintConsoleLine};
use bevy_egui::egui::style::{Widgets, WidgetVisuals};
use bevy_file_dialog::FileDialogPlugin;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::egui;
use bevy_inspector_egui::egui::{FontData, FontFamily, Stroke};
use bevy_inspector_egui::egui::epaint::Shadow;
use clap::builder::StyledStr;
use color_print::cformat;
use imageproc::drawing::Canvas;
use lazy_static::lazy_static;
use log::LevelFilter;
use winit::window::Icon;

use settings::Settings;

use crate::common::{BufferedLogger, Coordinates, LogMessage};
use crate::common::io::{Load, Save};
use crate::editor_data::{EditorData, IntoColor32};
use crate::editor_data::io::{EditorDataLoader, EditorDataSaver};
use crate::graphics::GraphicsResource;
use crate::map::events::{ClearTiles, SpawnMapEntity};
use crate::map::MapPlugin;
use crate::map::resources::MapEntity;
use crate::map::systems::{set_tile_reader, spawn_sprite, tile_despawn_reader, tile_remove_reader, tile_spawn_reader, update_sprite_reader};
use crate::project::resources::Project;
use crate::settings::{SettingsLoader, SettingsSaver};
use crate::tiles::components::{Offset, Tile};
use crate::tiles::TilePlugin;
use crate::ui::components::CDDADirContents;
use crate::ui::grid::{GridMaterial, GridPlugin};
use crate::ui::grid::resources::Grid;
use crate::ui::interaction::{CDDADirPicked, TilesetSelected};
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
mod settings;

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
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "CDDA Map Editor".to_string(),
                    mode: WindowMode::Windowed,
                    ..Default::default()
                }),
                ..Default::default()
            }).disable::<LogPlugin>(), ConsolePlugin)
        )
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
        ))
        .add_systems(PostStartup, (
            spawn_initial_tabs,
            setup_egui,
        ))
        .add_event::<SwitchProject>()
        .add_plugins(FileDialogPlugin::new()
            .with_save_file::<Project>()
            .with_load_file::<Project>()
            .with_pick_directory::<CDDADirContents>()
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
            exit_system
        ).chain())
        .run();
}

fn spawn_initial_tabs(
    mut e_spawn_tab: EventWriter<SpawnTab>,
    r_editor_data: Res<EditorData>,
) {
    for (i, project) in r_editor_data.projects.iter().enumerate() {
        let name = match &project.map_entity {
            MapEntity::Single(s) => s.om_terrain.clone(),
            MapEntity::Nested(_) => "Nested_TODO".to_string(),
            _ => todo!()
        };

        e_spawn_tab.send(SpawnTab { name, index: i as u32 });
    }
}

fn setup(
    mut commands: Commands,
    mut e_cdda_dir_picked: EventWriter<CDDADirPicked>,
    mut e_tileset_selected: EventWriter<TilesetSelected>,
    win_windows: NonSend<WinitWindows>,
) {
    commands.spawn(Camera2dBundle::default());

    let settings = match (SettingsLoader {}.load()) {
        Ok(s) => {
            e_cdda_dir_picked.send(CDDADirPicked {
                path: s.selected_cdda_dir.clone()
            });

            e_tileset_selected.send(TilesetSelected {});

            s
        }
        Err(_) => Settings::default()
    };

    commands.insert_resource(settings);

    let editor_data_io = EditorDataLoader::new();
    let editor_data = editor_data_io.load().unwrap();

    let texture_resource = GraphicsResource::default();

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

    commands.insert_resource(editor_data);
    commands.insert_resource(texture_resource);
}

fn setup_egui(
    mut contexts: EguiContexts,
    r_editor_data: ResMut<EditorData>,
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
        widgets: Widgets {
            open: WidgetVisuals {
                bg_fill: Default::default(),
                weak_bg_fill: r_editor_data.config.style.blue_dark.into_color32(),
                bg_stroke: Default::default(),
                rounding: Default::default(),
                fg_stroke: Default::default(),
                expansion: 0.0,
            },
            ..Default::default()
        },
        extreme_bg_color: r_editor_data.config.style.blue_dark.into_color32(),
        window_stroke: Stroke::NONE,
        override_text_color: Some(r_editor_data.config.style.white.into_color32()),
        window_fill: r_editor_data.config.style.gray_darker.into_color32(),
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

    if let Some(grid_material) = grid_material.iter_mut().next() {
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
            transform.translation.x = (-window.resolution.width() / 2. + res_grid.tile_size / 2.) - (res_grid.offset.x - coordinates.x as f32 * res_grid.tile_size);
            transform.translation.y = (window.resolution.height() / 2. - (res_grid.tile_size + (sprite_offset.y as f32 * (res_grid.tile_size / res_grid.default_tile_size))) / 2.) + (res_grid.offset.y - coordinates.y as f32 * res_grid.tile_size);
        }
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

        r_editor_data.current_project_index = Some(switch_project.index);
    }
}

fn exit_system(
    e_exit: EventReader<AppExit>,
    r_settings: Res<Settings>,
    r_editor_data: Res<EditorData>,
) {
    if e_exit.is_empty() { return; }

    let data_saver = EditorDataSaver::new();
    data_saver.save(&r_editor_data).unwrap();

    let settings_saver = SettingsSaver {};
    settings_saver.save(&r_settings).unwrap();
}