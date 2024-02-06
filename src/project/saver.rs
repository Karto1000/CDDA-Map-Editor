use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::project::Project;

#[derive(Debug)]
pub enum SaveError {
    DirectoryNotFound(String),
    InvalidPath(anyhow::Error),
}

pub trait Save<T> {
    fn save(&self, value: &T) -> Result<(), SaveError>;
}

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
        let mut file = match File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.directory.join(format!("auto_save_{}.map", value.map_entity.om_terrain))) {
            Ok(f) => f,
            Err(e) => return Err(SaveError::InvalidPath(e.into()))
        };

        file.write_all(serde_json::to_string(value).unwrap().as_bytes()).unwrap();

        return Ok(());
    }
}