use bevy::app::AppExit;
use bevy::prelude::{Changed, Commands, EventWriter, Interaction, Query, Res, With};
use bevy_file_dialog::FileDialogExt;

use crate::hotbar::systems::{CloseIconMarker, ImportIconMarker, SaveIconMarker};
use crate::map::resources::MapEntity;
use crate::project::{EditorData, Project};

pub fn close_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<CloseIconMarker>)>,
    mut exit: EventWriter<AppExit>,
) {
    for interaction in interaction_query.iter() {
        match interaction {
            Interaction::Pressed => { exit.send(AppExit) }
            _ => {}
        };
    }
}

pub fn save_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SaveIconMarker>)>,
    res_editor_data: Res<EditorData>,
    mut commands: Commands,
) {
    let project = res_editor_data.get_current_project();

    for interaction in interaction_query.iter() {
        match interaction {
            Interaction::Pressed => {
                let map_json = serde_json::to_string(&project.map_entity).unwrap();

                commands.dialog()
                    .set_file_name(format!("{}.map", project.map_entity.name))
                    .save_file::<MapEntity>(map_json.as_bytes().into());
            }
            _ => {}
        };
    }
}

pub fn import_button_interaction(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ImportIconMarker>)>,
    mut commands: Commands,
) {
    for interaction in interaction_query.iter() {
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