use std::fs;

use bevy::hierarchy::Children;
use bevy::input::Input;
use bevy::prelude::{Commands, Entity, EventReader, KeyCode, Query, Res, ResMut, Text};
use bevy::text::TextSection;
use bevy_file_dialog::{DialogFileSaved, FileDialogExt};

use crate::EditorData;
use crate::map::{TileDeleteEvent, TilePlaceEvent};
use crate::project::resources::{Project, ProjectSaveState};
use crate::ui::tabs::components::Tab;

pub fn map_save_system(
    keys: Res<Input<KeyCode>>,
    r_editor_data: ResMut<EditorData>,
    mut commands: Commands,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::S) {
        let current_project = r_editor_data.get_current_project().unwrap();

        match &current_project.save_state {
            ProjectSaveState::Saved(p) => {
                fs::write(p, serde_json::to_string(&current_project).unwrap().into_bytes()).unwrap();
            }
            _ => {
                commands.dialog()
                    .set_file_name("unnamed.map")
                    .save_file::<Project>(serde_json::to_string(&current_project).unwrap().into_bytes());
            }
        }
    }
}

pub fn save_directory_picked(
    mut res_editor_data: ResMut<EditorData>,
    mut e_file_saved: EventReader<DialogFileSaved<Project>>,
    q_tabs: Query<(Entity, &Tab, &Children)>,
    mut q_text: Query<&mut Text>,
) {
    let project_index = res_editor_data.current_project_index;
    let current_project = match res_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    for event in e_file_saved.read() {
        current_project.save_state = ProjectSaveState::Saved(event.path.clone());
        current_project.map_entity.om_terrain = event.path.file_name().unwrap().to_str().unwrap().to_string();

        // Edit the file name in the saved file because we can't know the file name in advance
        let content = fs::read_to_string(&event.path).unwrap();
        let mut entity: Project = serde_json::from_str(content.as_str()).unwrap();

        // This is probably some of the weirdest code i've ever written
        let file_name_string = event.path
            .file_name()
            .clone()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let reversed_string = file_name_string
            .chars()
            .rev()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("");

        let project_name = reversed_string
            // Remove the extension with the dot
            .splitn(2, ".")
            .last()
            .unwrap()
            .chars()
            .rev()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join("");

        for (_, tab, children) in q_tabs.iter() {
            if tab.index != project_index { continue; }

            for child in children.iter() {
                let mut text = match q_text.get_mut(*child) {
                    Ok(t) => t,
                    Err(_) => { continue; }
                };

                text.sections.clear();
                text.sections.push(TextSection::from(project_name.clone()));
            }
        }

        entity.map_entity.om_terrain = project_name;
        entity.save_state = ProjectSaveState::Saved(event.path.clone());

        // Remove the original file and Save it back and overwrite the original file
        fs::remove_file(&event.path).unwrap();
        fs::write(&event.path, serde_json::to_string(&entity).unwrap().into_bytes()).unwrap();
    }
}

pub fn set_tile_reader(
    mut e_set_tile: EventReader<TilePlaceEvent>,
    mut r_editor_data: ResMut<EditorData>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_set_tile.read() {
        project.map_entity.tiles.insert(
            e.coordinates.clone(),
            e.tile,
        );
    }
}

pub fn tile_remove_reader(
    mut e_delete_tile: EventReader<TileDeleteEvent>,
    mut r_editor_data: ResMut<EditorData>,
) {
    let project = match r_editor_data.get_current_project_mut() {
        None => { return; }
        Some(p) => { p }
    };

    for e in e_delete_tile.read() {
        project.map_entity.tiles.remove(&e.coordinates);
    }
}