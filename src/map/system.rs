use bevy::input::Input;
use bevy::prelude::{Commands, KeyCode, Res};
use bevy_file_dialog::FileDialogExt;

use crate::map::resources::MapEntity;

pub fn map_save_system(
    res_map: Res<MapEntity>,
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::S) {
        let map_json = res_map.export().unwrap();

        commands.dialog()
            .set_file_name(format!("{}.json", res_map.name))
            .save_file::<Vec<u8>>(map_json.to_string().as_bytes().into());
    }
}