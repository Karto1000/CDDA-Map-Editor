use bevy::app::AppExit;
use bevy::prelude::{Changed, Commands, EventReader, EventWriter, Interaction, Query, Res, ResMut, With};
use bevy_file_dialog::{DialogFileLoaded, FileDialogExt};

use crate::editor_data::EditorData;
use crate::map::resources::MapEntity;
use crate::project::resources::{Project, ProjectSaveState};
use crate::ui::hotbar::components::{CloseIconMarker, ImportIconMarker, OpenIconMarker, SaveIconMarker};
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