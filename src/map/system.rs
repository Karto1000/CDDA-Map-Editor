use std::fs::File;
use std::io::Write;

use bevy::input::Input;
use bevy::prelude::{KeyCode, Res};

use crate::map::MapEntity;

pub fn map_save_system(
    res_map: Res<MapEntity>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::S) {
        let map_json = res_map.json().unwrap();

        File::create(format!("saves/{}.json", res_map.name)).unwrap().write(map_json.to_string().as_bytes());
    }
}