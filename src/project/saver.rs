use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::common::io::{Save, SaveError};
use crate::project::resources::Project;



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
        let filename = format!("auto_save_{}.map", value.map_entity.map_type.get_name());

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