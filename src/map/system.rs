use std::fs;

use bevy::input::Input;
use bevy::prelude::{Commands, KeyCode, Res};
use directories::ProjectDirs;

use crate::project;
use crate::project::{EditorData, Project};
use crate::project::saver::Save;

pub fn map_save_system(
    res_editor_data: Res<EditorData>,
    keys: Res<Input<KeyCode>>,
) {
    let project = res_editor_data.get_current_project();

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::S) {
        // let map_json = res_map.export().unwrap();
        //
        // commands.dialog()
        //     .set_file_name(format!("{}.json", res_map.name))
        //     .save_file::<Vec<u8>>(map_json.to_string().as_bytes().into());

        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return; }
            Some(d) => d
        };

        let auto_save_dir = dir.data_local_dir();

        if !auto_save_dir.exists() { fs::create_dir_all(auto_save_dir).unwrap(); }

        let saver = project::saver::ProjectSaver::new(auto_save_dir.into()).unwrap();
        println!("{:?}", saver.save(project));
    }
}