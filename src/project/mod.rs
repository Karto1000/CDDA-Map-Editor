use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bevy::prelude::{Resource, Vec2};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::map::resources::MapEntity;
use crate::project::loader::{Load, LoadError};
use crate::project::saver::{Save, SaveError};

pub(crate) mod saver;
pub(crate) mod loader;

#[derive(Debug, Default, Resource, Serialize, Deserialize)]
pub struct Project {
    pub map_entity: MapEntity,
    pub map_save_path: Option<Box<Path>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectSaveState {
    /// If the Project was saved somewhere
    /// Contains the path to the saved project
    Saved(Box<Path>),

    /// If the project was not saved
    /// Contains the path to an auto save file (if one exists)
    NotSaved(Option<Box<Path>>),
}

/// Struct that stores the data that should be saved permanently in the users app data directory
#[derive(Debug, Resource, Serialize, Deserialize)]
pub struct EditorData {
    pub current_project_index: u32,
    pub projects: Vec<Project>,
    pub history: Vec<ProjectSaveState>,
}

impl EditorData {
    pub fn get_current_project(&self) -> &Project {
        return self.projects.get(self.current_project_index as usize).unwrap();
    }

    pub fn get_current_project_mut(&mut self) -> &mut Project {
        return self.projects.get_mut(self.current_project_index as usize).unwrap();
    }
}

impl Default for EditorData {
    fn default() -> Self {
        let map: MapEntity = MapEntity::new(
            "unnamed".into(),
            Vec2::new(24., 24.),
        );

        let project = Project {
            map_entity: map,
            map_save_path: None,
        };

        return Self {
            current_project_index: 0,
            projects: vec![project],
            history: vec![],
        };
    }
}

pub struct EditorDataSaver;

impl Save<EditorData> for EditorDataSaver {
    fn save(&self, value: &EditorData) -> Result<(), SaveError> {
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
            .open(data_dir.join("data.json"))
            .unwrap();

        let mut converted = serde_json::to_value(value).unwrap();
        let data = converted.as_object_mut().unwrap();

        let open_projects: Vec<ProjectSaveState> = value.projects.iter().map(|project| {
            return match &project.map_save_path {
                None => ProjectSaveState::NotSaved(Some(data_dir.join(format!("auto_save_{}.json", project.map_entity.name)).into_boxed_path())),
                Some(p) => ProjectSaveState::Saved(p.clone())
            };
        }).collect();

        data.insert("open_projects".into(), serde_json::to_value(open_projects).unwrap());

        data.remove("projects".into());
        data.remove("current_project_index".into());

        file.write_all(serde_json::to_string(data).unwrap().as_bytes()).unwrap();

        return Ok(());
    }
}

impl Load<EditorData> for EditorDataSaver {
    fn load(&self) -> Result<EditorData, LoadError> {
        todo!()
    }
}