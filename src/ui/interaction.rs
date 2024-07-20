use std::fs;
use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::prelude::{Assets, Changed, Commands, Entity, Event, EventReader, EventWriter, Image, Interaction, Query, Res, ResMut, State, With};
use bevy_egui::egui;
use bevy_egui::egui::{Align, Color32, Margin, Ui, WidgetText};
use bevy_file_dialog::{DialogDirectoryPicked, DialogFileLoaded, FileDialogExt};
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::graphics::{GraphicsResource, LegacyTextures};
use crate::graphics::tileset::legacy::LegacyTilesetLoader;
use crate::map::data::MapEntity;
use crate::program::data::{IntoColor32, OpenedProject, Program, ProgramState};
use crate::project::data::{Project, ProjectSaveState};
use crate::region_settings::io::RegionSettingsLoader;
use crate::settings::data::Settings;
use crate::ui::CDDADirContents;
use crate::ui::egui_utils::add_settings_frame;
use crate::ui::hotbar::components::{CloseIconMarker, ImportIconMarker, OpenIconMarker, SaveIconMarker, SettingsIconMarker, TileSettingsMarker};
use crate::ui::tabs::events::SpawnTab;
use crate::ui::terrain::TerrainMenuData;
use crate::program::data::Menus;

pub fn close_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseIconMarker>)>,
    mut exit: EventWriter<AppExit>,
) {
    for interaction in interaction_query.iter() {
        match interaction {
            Interaction::Pressed => { exit.send(AppExit); }
            _ => {}
        };
    }
}

pub fn save_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SaveIconMarker>)>,
    r_program: Res<Program>,
    s_state: Res<State<ProgramState>>,
    mut commands: Commands,
    q_opened_project: Query<(Entity, &OpenedProject)>,
) {
    let index = match q_opened_project.iter().next() {
        None => return,
        Some(o) => o.1.index
    };

    let project = match r_program.projects.get(index) {
        None => return,
        Some(p) => p
    };

    let filename = match &project.map_entity {
        MapEntity::Single(s) => s.om_terrain.clone(),
        MapEntity::Multi(_) => todo!(),
        MapEntity::Nested(_) => "Nested_TODO".to_string()
    };

    for interaction in interaction_query.iter() {
        match interaction {
            Interaction::Pressed => {
                let project_json = serde_json::to_string(&project).unwrap();
                commands.dialog()
                    .set_file_name(filename.clone())
                    .save_file::<Project>(project_json.into_bytes());
            }
            _ => {}
        };
    }
}

pub fn import_button_interaction(
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<ImportIconMarker>)>,
    mut commands: Commands,
) {
    for interaction in q_interaction.iter() {
        match interaction {
            Interaction::Pressed => {
                commands.dialog()
                    .add_filter("", vec!["map"].as_slice())
                    .load_file::<MapEntity>();
            }
            _ => {}
        };
    }
}

pub fn open_button_interaction(
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<OpenIconMarker>)>,
    mut commands: Commands,
) {
    for interaction in q_interaction.iter() {
        match interaction {
            Interaction::Pressed => {
                commands.dialog()
                    .add_filter("", &*vec!["map"])
                    .load_file::<Project>()
            }
            _ => {}
        }
    }
}

pub fn file_loaded_reader(
    mut e_file_loaded: EventReader<DialogFileLoaded<Project>>,
    mut e_spawn_tab: EventWriter<SpawnTab>,
    mut r_program: ResMut<Program>,
) {
    for event in e_file_loaded.read() {
        if r_program.projects.iter().any(|p| {
            // Make it so the same project can't be opened more than once
            match &p.save_state {
                ProjectSaveState::Saved(path) => path.clone() == event.path,
                ProjectSaveState::AutoSaved(path) => path.clone() == event.path,
                _ => { return false; }
            }
        }) == true {
            return;
        };

        let project = serde_json::from_slice::<Project>(event.contents.as_slice()).unwrap();

        let name = match &project.map_entity {
            MapEntity::Single(s) => s.om_terrain.clone(),
            MapEntity::Multi(_) => todo!(),
            MapEntity::Nested(_) => todo!()
        };

        e_spawn_tab.send(SpawnTab {
            name,
            index: r_program.projects.len() as u32,
        });

        r_program.projects.push(project);
    }
}

#[derive(Event, Debug)]
pub struct CDDADirPicked {
    pub path: PathBuf,
}

pub fn file_dialog_cdda_dir_picked(
    mut e_dialog_cdda_dir_picked: EventReader<DialogDirectoryPicked<CDDADirContents>>,
    mut e_cdda_dir_picked: EventWriter<CDDADirPicked>,
) {
    for e in e_dialog_cdda_dir_picked.read() {
        e_cdda_dir_picked.send(CDDADirPicked {
            path: e.path.clone()
        });
    }
}

pub fn cdda_folder_picked(
    mut e_cdda_dir_picked: EventReader<CDDADirPicked>,
    mut r_settings: ResMut<Settings>,
    mut r_program: ResMut<Program>,
) {
    for e in e_cdda_dir_picked.read() {
        r_settings.selected_cdda_dir = Some(e.path.clone());
        r_program.config.load_cdda_dir(e.path.clone());

        fs::read_dir(&e.path.join("gfx")).unwrap().into_iter().for_each(|e| {
            match e {
                Ok(e) => {
                    if !e.path().is_dir() { return; }

                    if r_settings.selectable_tilesets.contains(&e.file_name().to_str().unwrap().to_string()) {
                        return;
                    }

                    r_settings.selectable_tilesets.push(e.file_name().to_str().unwrap().to_string());
                }
                Err(_) => {}
            };
        });
    }
}

#[derive(Debug, Event)]
pub struct TilesetSelected {
    pub name: String,
}

pub fn tileset_selected(
    mut e_tileset_selected: EventReader<TilesetSelected>,
    r_settings: Res<Settings>,
    mut r_graphics_resource: ResMut<GraphicsResource>,
    mut r_images: ResMut<Assets<Image>>,
) {
    match &r_settings.selected_cdda_dir {
        None => return,
        Some(_) => {}
    };

    for e in e_tileset_selected.read() {
        let tileset_loader = LegacyTilesetLoader::new(r_settings.gfx_dir().unwrap().join(e.name.clone()));
        let region_settings_loader = RegionSettingsLoader::new(
            r_settings.data_json_dir().unwrap().join(r"regional_map_settings.json"),
            "default".into(),
        );

        let textures = LegacyTextures::new(
            tileset_loader,
            region_settings_loader,
            &mut r_images,
        );

        r_graphics_resource.textures = Some(Box::new(textures));
    }
}

pub fn define_terrain_button_interaction(
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<TileSettingsMarker>)>,
    r_program_state: Res<State<ProgramState>>,
    mut r_program: ResMut<Program>,
    mut r_menus: ResMut<Menus>,
    mut commands: Commands
) {
    match r_program_state.get() {
        ProgramState::ProjectOpen => {},
        _ => return
    }
    
    for interaction in q_interaction.iter() {
        match interaction {
            Interaction::Pressed => {
                match r_menus.is_define_terrain_menu_open {
                    false => commands.insert_resource(TerrainMenuData::default()),
                    true => commands.remove_resource::<TerrainMenuData>()
                };
                r_menus.is_define_terrain_menu_open = !r_menus.is_define_terrain_menu_open;
            },
            _ => {}
        }
    }
}

pub fn settings_button_interaction(
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<SettingsIconMarker>)>,
    mut contexts: EguiContexts,
    mut r_program: ResMut<Program>,
    mut r_menus: ResMut<Menus>,
    mut commands: Commands,
    mut r_settings: ResMut<Settings>,
    mut e_tileset_selected: EventWriter<TilesetSelected>,
) {
    let gray_dark_color32 = r_program.config.style.gray_dark.into_color32();

    egui::Window::new("General Settings")
        .open(&mut r_menus.is_settings_menu_open)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                ui.label("Here you can change the general settings of the editor which applies to all Projects");
                ui.set_max_width(500.);

                add_settings_frame(
                    "General",
                    gray_dark_color32,
                    ui,
                    |ui| {
                        let mut string = match &r_settings.selected_cdda_dir {
                            Some(v) => v.to_str().unwrap().to_string(),
                            None => "".into()
                        };

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.;

                            let text_edit = egui::widgets::TextEdit::singleline(&mut string)
                                .vertical_align(Align::Center)
                                .desired_width(0.);

                            let response = ui.add_sized([ui.available_width(), 32.], text_edit);

                            if response.clicked() {
                                commands.dialog()
                                    .pick_directory_path::<CDDADirContents>()
                            }

                            ui.label("CDDA Directory");
                        });
                    },
                );

                add_settings_frame(
                    "Tile Settings",
                    gray_dark_color32,
                    ui,
                    |ui| {
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_source(0)
                                .selected_text(r_settings.selected_tileset.as_ref().unwrap_or(&"".into()))
                                .show_ui(ui, |ui| {
                                    let entries = r_settings.selectable_tilesets.clone();
                                    for tileset_name in entries.into_iter() {
                                        let value = ui.selectable_value(
                                            &mut r_settings.selected_tileset,
                                            Some(tileset_name.clone()),
                                            tileset_name.clone(),
                                        );

                                        if value.clicked() {
                                            e_tileset_selected.send(TilesetSelected {
                                                name: tileset_name
                                            });
                                        }
                                    }
                                });
                        });
                    },
                );
            });
        });

    for interaction in q_interaction.iter() {
        match interaction {
            Interaction::Pressed => {
                r_menus.is_settings_menu_open = !r_menus.is_settings_menu_open;
            }
            _ => {}
        }
    }
}