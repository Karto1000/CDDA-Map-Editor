use std::fs;

use bevy::app::AppExit;
use bevy::prelude::{Changed, Commands, EventReader, EventWriter, Interaction, Query, Res, ResMut, With};
use bevy_egui::egui;
use bevy_egui::egui::{Align, Color32, Margin, Ui, WidgetText};
use bevy_file_dialog::{DialogDirectoryPicked, DialogFileLoaded, FileDialogExt};
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::editor_data::{EditorData, IntoColor32};
use crate::map::resources::MapEntity;
use crate::project::resources::{Project, ProjectSaveState};
use crate::ui::components::CDDADirContents;
use crate::ui::hotbar::components::{CloseIconMarker, ImportIconMarker, OpenIconMarker, SaveIconMarker, SettingsIconMarker};
use crate::ui::settings::Settings;
use crate::ui::tabs::events::SpawnTab;

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
    r_editor_data: Res<EditorData>,
    mut commands: Commands,
) {
    let project = match r_editor_data.get_current_project() {
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
    mut r_editor_data: ResMut<EditorData>,
) {
    for event in e_file_loaded.read() {
        if r_editor_data.projects.iter().any(|p| {
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
            index: r_editor_data.projects.len() as u32,
        });

        r_editor_data.projects.push(project);
    }
}

pub fn cdda_folder_picked(
    mut e_cdda_dir_picked: EventReader<DialogDirectoryPicked<CDDADirContents>>,
    mut r_settings: ResMut<Settings>,
    mut r_editor_data: ResMut<EditorData>,
) {
    for e in e_cdda_dir_picked.read() {
        r_settings.selected_cdda_dir = e.path.clone();
        r_editor_data.config.load_cdda_dir(r_settings.selected_cdda_dir.clone());

        fs::read_dir(&e.path.join("gfx")).unwrap().into_iter().for_each(|e| {
            match e {
                Ok(e) => {
                    if e.path().is_dir() {
                        r_settings.selectable_tilesets.push(e.file_name().to_str().unwrap().to_string())
                    }
                }
                Err(_) => {}
            };
        });
    }
}

pub fn settings_button_interaction(
    q_interaction: Query<&Interaction, (Changed<Interaction>, With<SettingsIconMarker>)>,
    mut contexts: EguiContexts,
    mut r_editor_data: ResMut<EditorData>,
    mut commands: Commands,
    mut r_settings: ResMut<Settings>,
) {
    let gray_dark_color32 = r_editor_data.config.style.gray_dark.into_color32();

    egui::Window::new("General Settings")
        .open(&mut r_editor_data.menus.is_settings_menu_open)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical(|ui| {
                ui.label("Here you can change the general settings of the editor which applies to all Projects");
                ui.set_max_width(500.);

                fn add_settings_menu(
                    name: impl Into<WidgetText>,
                    fill: Color32, ui: &mut Ui,
                    add: impl FnOnce(&mut Ui),
                ) {
                    egui::Frame::none()
                        .fill(fill)
                        .inner_margin(Margin::same(8.))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            ui.spacing_mut().item_spacing.y = 12.;
                            ui.vertical(|ui| {
                                ui.label(name);
                                add(ui);
                            });
                        });
                }

                add_settings_menu(
                    "General",
                    gray_dark_color32,
                    ui,
                    |ui| {
                        let mut string = r_settings.selected_cdda_dir.to_str().unwrap().to_string();

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

                add_settings_menu(
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
                                        ui.selectable_value(&mut r_settings.selected_tileset, Some(tileset_name.clone()), tileset_name.clone());
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
                r_editor_data.menus.is_settings_menu_open = !r_editor_data.menus.is_settings_menu_open;
            }
            _ => {}
        }
    }
}