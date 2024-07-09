use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use directories::ProjectDirs;

use crate::common::io::{Load, LoadError, Save, SaveError};
use crate::common::io::LoadError::NoAutoSave;
use crate::map::data::MapEntity;
use crate::project::data::Project;

pub struct ProjectSaver {
    pub directory: Box<Path>,
}

impl ProjectSaver {
    pub fn new(directory: Box<Path>) -> Result<Self, SaveError> {
        if !directory.exists() {
            return Err(SaveError::DirectoryNotFound(directory.to_str().unwrap_or("UNKNOWN").to_string()));
        }

        return Ok(Self { directory });
    }
}

impl Save<Project> for ProjectSaver {
    fn save(&self, value: &Project) -> Result<(), SaveError> {
        let filename = match &value.map_entity {
            MapEntity::Single(s) => s.om_terrain.clone(),
            _ => todo!()
        };

        let filename = format!("auto_save_{}.map", filename);

        let mut file = match File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.directory.join(filename)) {
            Ok(f) => f,
            Err(e) => return Err(SaveError::InvalidPath(e.into()))
        };

        file.write_all(serde_json::to_string(value).unwrap().as_bytes()).unwrap();

        return Ok(());
    }
}

pub struct ProjectAutoSaveLoader {
    directory: Box<Path>,
    map_name: String,
}

impl ProjectAutoSaveLoader {
    pub fn new(map_name: String) -> Result<Self, LoadError> {
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(LoadError::DirectoryNotFound); }
            Some(d) => d
        };

        let auto_save_dir = dir.data_local_dir();

        if !auto_save_dir.exists() { fs::create_dir_all(auto_save_dir).unwrap(); }

        return Ok(Self {
            directory: auto_save_dir.into(),
            map_name,
        });
    }
}

impl Load<Project> for ProjectAutoSaveLoader {
    fn load(&self) -> Result<Project, LoadError> {
        if !self.directory.exists() {
            return Err(LoadError::DirectoryNotFound);
        }

        let mut file = match File::open(self.directory.join(format!("auto_save_{}.map", self.map_name))) {
            Ok(f) => f,
            Err(_) => return Err(NoAutoSave)
        };

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();

        return Ok(serde_json::from_slice(contents.as_slice()).unwrap());
    }
}
