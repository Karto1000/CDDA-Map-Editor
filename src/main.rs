use std::default::Default;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::string::ToString;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy::app::{App, AppExit, PluginGroup};
use bevy::asset::AsyncReadExt;
use bevy::DefaultPlugins;
use bevy::log::LogPlugin;
use bevy::prelude::{Assets, Camera2dBundle, Commands, EventReader, NonSend, Query, Res, ResMut, Transform, Vec2, Window, With};
use bevy::sprite::Material2dPlugin;
use bevy::utils::default;
use bevy::window::{WindowMode, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy_console::{ConsoleConfiguration, ConsolePlugin, PrintConsoleLine};
use bevy_egui::egui::style::{Widgets, WidgetVisuals};
use bevy_egui::EguiPlugin;
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

use program::data::IntoColor32;
use project::data::Project;
use settings::data::Settings;
use settings::io::{SettingsLoader, SettingsSaver};
use tiles::data::{Offset, Tile};
use ui::{CDDADirContents, IsCursorCaptured};

use crate::common::{BufferedLogger, Coordinates, LogMessage};
use crate::common::io::{Load, Save};
use crate::graphics::GraphicsResource;
use crate::map::data::MapEntity;
use crate::map::io::MapEntityLoader;
use crate::map::plugin::MapPlugin;
use crate::map::systems::{clear_tiles_reader, set_tile_reader, spawn_map_entity_reader, spawn_sprite, tile_despawn_reader, tile_remove_reader, tile_spawn_reader, update_sprite_reader};
use crate::program::data::{Menus, OpenedProject, Program, ProgramState};
use crate::program::io::{ProgramdataLoader, ProgramdataSaver};
use crate::program::plugin::ProgramPlugin;
use crate::project::data::CreateProject;
use crate::project::plugin::ProjectPlugin;
use crate::tiles::plugin::TilePlugin;
use crate::ui::grid::GridMaterial;
use crate::ui::grid::GridPlugin;
use crate::ui::grid::resources::Grid;
use crate::ui::interaction::{cdda_folder_picked, CDDADirPicked, close_button_interaction, TilesetSelected};
use crate::ui::minimap::plugin::MinimapPlugin;
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
mod settings;
mod program;

lazy_static! {
    pub static ref LOGGER: BufferedLogger = BufferedLogger::new();
}

fn main() {
    lazy_static::initialize(&LOGGER);
    log::set_logger(LOGGER.deref()).unwrap();
    log::set_max_level(LevelFilter::Info);

    let mut app = App::new();

    // -- Add Plugins --
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CDDA Map Editor".to_string(),
                mode: WindowMode::Windowed,
                ..Default::default()
            }),
            ..Default::default()
        }).disable::<LogPlugin>(),
        ConsolePlugin,
        FileDialogPlugin::new()
            .with_save_file::<Project>()
            .with_load_file::<Project>()
            .with_pick_directory::<CDDADirContents>(),
        Material2dPlugin::<GridMaterial>::default(),
        GridPlugin,
        MapPlugin,
        TilePlugin,
        UiPlugin,
        ProgramPlugin,
        ProjectPlugin,
        MinimapPlugin
    ));

    // -- Add Resources --
    app.insert_resource(Menus::default());

    // -- Add Events --
    app.add_event::<LogMessage>();

    // -- Add Systems --

    // Startup
    app.add_systems(Startup, (setup, apply_deferred, cdda_folder_picked, apply_deferred, load).chain());

    // Post Startup
    app.add_systems(PostStartup, (setup_egui));

    // Update
    let sys = (
        update,
        tile_despawn_reader,
        apply_deferred,
        tile_remove_reader,
        apply_deferred,
        spawn_map_entity_reader,
        apply_deferred,
        clear_tiles_reader,
        apply_deferred,
        set_tile_reader,
        tile_spawn_reader,
        apply_deferred,
        spawn_sprite,
        apply_deferred,
        update_sprite_reader,
        close_button_interaction,
        apply_deferred,
        exit,
    );

    app.add_systems(Update, sys.chain());

    app.run();
}

fn load(
    r_program: Res<Program>,
    mut e_create_project: EventWriter<CreateProject>,
    mut e_spawn_tab: EventWriter<SpawnTab>
) {
    let loader = MapEntityLoader {
        path: PathBuf::from(r"C:\CDDA\testing\data\json\mapgen\basic\field.json"),
        id: "field".into(),
        cdda_data: r_program.config.cdda_data.as_ref().unwrap(),
    };

    let entity: MapEntity = MapEntity::Single(loader.load().unwrap());

    let project = Project {
        name: "Field".into(),
        map_entity: entity,
        save_state: Default::default(),
    };

    e_create_project.send(CreateProject {
        project
    });

    e_spawn_tab.send(SpawnTab {
        name: "Field".into(),
        index: 0,
    });
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
            if let Some(path) = &s.selected_cdda_dir {
                e_cdda_dir_picked.send(CDDADirPicked {
                    path: path.clone()
                });
            }

            if let Some(name) = &s.selected_tileset {
                e_tileset_selected.send(TilesetSelected {
                    name: name.clone()
                });
            }

            s
        }
        Err(_) => Settings::default()
    };

    let program_loader = ProgramdataLoader {};
    let program_data = program_loader.load().unwrap_or(Program::new(vec![], vec![]));

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

    commands.insert_resource(ConsoleConfiguration {
        keys: program_data.config.keybindings.open_console.clone(),
        ..default()
    });
    commands.insert_resource(settings);
    commands.insert_resource(ClearColor(program_data.config.style.gray_dark.clone()));
    commands.insert_resource(program_data);
    commands.insert_resource(texture_resource);
}

fn setup_egui(
    mut contexts: EguiContexts,
    r_program: ResMut<Program>,
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
                weak_bg_fill: r_program.config.style.blue_dark.into_color32(),
                bg_stroke: Default::default(),
                rounding: Default::default(),
                fg_stroke: Default::default(),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                bg_fill: r_program.config.style.white.into_color32(),
                weak_bg_fill: r_program.config.style.blue_dark.into_color32(),
                bg_stroke: Default::default(),
                rounding: Default::default(),
                fg_stroke: Stroke::new(1., r_program.config.style.white.into_color32()),
                expansion: 0.0,
            },
            hovered: WidgetVisuals {
                bg_fill: Default::default(),
                weak_bg_fill: r_program.config.style.selected.into_color32(),
                bg_stroke: Default::default(),
                rounding: Default::default(),
                fg_stroke: Stroke::new(1., r_program.config.style.white.into_color32()),
                expansion: 0.0,
            },
            active: WidgetVisuals {
                bg_fill: Default::default(),
                weak_bg_fill: r_program.config.style.selected.into_color32(),
                bg_stroke: Default::default(),
                rounding: Default::default(),
                fg_stroke: Stroke::new(1., r_program.config.style.white.into_color32()),
                expansion: 0.0,
            },
            ..Default::default()
        },
        extreme_bg_color: r_program.config.style.blue_dark.into_color32(),
        window_stroke: Stroke::NONE,
        override_text_color: Some(r_program.config.style.white.into_color32()),
        window_fill: r_program.config.style.gray_darker.into_color32(),
        ..default()
    });
}

fn update(
    r_grid: Res<Grid>,
    r_cursor: Res<IsCursorCaptured>,
    r_program: Res<Program>,
    r_program_state: Res<State<ProgramState>>,
    mut r_grid_material: ResMut<Assets<GridMaterial>>,
    mut q_tiles: Query<(&mut Transform, &Coordinates, &Offset), With<Tile>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut e_write_line: EventWriter<PrintConsoleLine>,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    for log in LOGGER.log_queue.read().unwrap().iter() {
        e_write_line.send(PrintConsoleLine::new(StyledStr::from(cformat!(r#"<g>[{}] {}</g>"#, log.level.as_str(), log.message))));
    }
    LOGGER.log_queue.write().unwrap().clear();

    if let Some(grid_material) = r_grid_material.iter_mut().next() {
        let index = match q_opened_project.iter().next() {
            None => return,
            Some(o) => o.1.index
        };

        let project = match r_program.projects.get(index) {
            None => return,
            Some(p) => p
        };

        let window = q_windows.single();

        grid_material.1.offset = r_grid.offset;
        grid_material.1.tile_size = r_grid.tile_size;
        grid_material.1.mouse_pos = window.cursor_position().unwrap_or(Vec2::default());
        grid_material.1.map_size = project.map_entity.size();
        // Weird way to do this but bevy does not let me pass a bool as a uniform for some reason
        grid_material.1.is_cursor_captured = match r_cursor.0 {
            true => 1,
            false => 0
        };
        grid_material.1.scale_factor = window.resolution.scale_factor();

        for (mut transform, coordinates, sprite_offset) in q_tiles.iter_mut() {
            transform.translation.x = (-window.resolution.width() / 2. + r_grid.tile_size / 2.) - (r_grid.offset.x - coordinates.x as f32 * r_grid.tile_size);
            transform.translation.y = (window.resolution.height() / 2. - (r_grid.tile_size + (sprite_offset.y as f32 * (r_grid.tile_size / r_grid.default_tile_size))) / 2.) + (r_grid.offset.y - coordinates.y as f32 * r_grid.tile_size);
        }
    }
}

fn exit(
    e_exit: EventReader<AppExit>,
    r_settings: Res<Settings>,
    r_editor_data: Res<Program>,
) {
    if e_exit.is_empty() { return; }

    let data_saver = ProgramdataSaver {};
    data_saver.save(&r_editor_data).unwrap();

    let settings_saver = SettingsSaver {};
    settings_saver.save(&r_settings).unwrap();
}