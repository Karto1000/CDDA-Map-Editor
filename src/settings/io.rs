use std::fs;
use std::fs::File;
use std::io::Write;

use directories::ProjectDirs;

use crate::common::io::{Load, LoadError, Save, SaveError};
use crate::settings::data::Settings;

pub struct SettingsLoader {}

impl Load<Settings> for SettingsLoader {
    fn load(&self) -> Result<Settings, LoadError> {
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(LoadError::DirectoryNotFound); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let contents = match fs::read_to_string(data_dir.join("settings.json")) {
            Err(e) => return Err(LoadError::Other(e.into())),
            Ok(f) => f
        };

        let settings: Settings = match serde_json::from_str::<Settings>(contents.as_str()) {
            Ok(s) => s,
            Err(e) => return Err(LoadError::Other(e.into()))
        };

        return Ok(settings);
    }
}

pub struct SettingsSaver {}

impl Save<Settings> for SettingsSaver {
    fn save(&self, value: &Settings) -> Result<(), SaveError> {
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(SaveError::DirectoryNotFound("".into())); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let mut file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(data_dir.join("settings.json"))
            .unwrap();

        match file.write_all(serde_json::to_string(&value).unwrap().as_bytes()) {
            Ok(_) => {}
            Err(e) => return Err(SaveError::Other(e.into()))
        }

        Ok(())
    }
}
