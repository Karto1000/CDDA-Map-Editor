use bevy::input::Input;
use bevy::prelude::{Commands, EventReader, KeyCode, Res, ResMut};
use bevy_file_dialog::{DialogFileSaved, FileDialogExt};

use crate::project;
use crate::project::{EditorData, Project};
use crate::project::saver::Save;

pub struct NoData;

pub fn map_save_system(
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::S) {
        commands.dialog().pick_directory_path::<NoData>();
    }
}

pub fn save_directory_picked(
    mut res_editor_data: ResMut<EditorData>,
    mut e_file_saved: EventReader<DialogFileSaved<Project>>
) {
    let project = match res_editor_data.get_current_project_mut() {
        None => return,
        Some(p) => p
    };

    for event in e_file_saved.read() {
        // TODO: REVISIT
        // Not able to be implemented because the event does not have a 'path' member
        // ISSUE HERE -> https://github.com/richardhozak/bevy_file_dialog/issues/1
        // project.map_save_path = Some(event.path);
    }
}