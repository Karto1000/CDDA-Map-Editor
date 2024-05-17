use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use directories::ProjectDirs;

use crate::common::io::{Load, LoadError};
use crate::common::io::LoadError::NoAutoSave;
use crate::project::resources::Project;

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