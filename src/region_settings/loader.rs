use std::fs;
use std::path::PathBuf;
use log::info;

use serde_json::Value;

use crate::common::io::{Load, LoadError};
use crate::region_settings::RegionSettings;

pub struct RegionSettingsLoader {
    pub path: PathBuf,
    pub id: String,
}

impl RegionSettingsLoader {
    pub fn new(path: PathBuf, id: String) -> Self {
        return Self {
            path,
            id
        }
    }
}

impl Load<RegionSettings> for RegionSettingsLoader {
    fn load(&self) -> Result<RegionSettings, LoadError> {
        let contents: Value = serde_json::from_str(fs::read_to_string(&self.path).unwrap().as_str()).unwrap();
        let contents_vec = contents.as_array().unwrap();

        let region_setting_object: RegionSettings = serde_json::from_value(
            contents_vec.iter()
                .find(|o| o.as_object().unwrap().get("id").unwrap().as_str().unwrap().to_string() == self.id)
                .unwrap()
                .clone()
        ).unwrap();

        info!("Successfully loaded RegionSettings '{}' at Path {:?}", self.id, self.path);

        return Ok(region_setting_object);
    }
}