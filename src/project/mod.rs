use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use bevy::prelude::{Resource, Vec2};
use directories::ProjectDirs;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    pub fn get_current_project(&self) -> Option<&Project> {
        return self.projects.get(self.current_project_index as usize);
    }

    pub fn get_current_project_mut(&mut self) -> Option<&mut Project> {
        return self.projects.get_mut(self.current_project_index as usize);
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
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(LoadError::DirectoryNotFound); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let contents = match fs::read_to_string(data_dir.join("data.json")) {
            Err(_) => return Ok(EditorData::default()),
            Ok(f) => f
        };

        let value: Map<String, Value> = serde_json::from_str(contents.as_str())
            .expect("Valid Json");

        let history_array: Vec<ProjectSaveState> = value
            .get("history")
            .expect("history Field")
            .as_array()
            .expect("Valid Array")
            .iter()
            .map(|v| serde_json::from_value::<ProjectSaveState>(v.clone()).unwrap())
            .collect();

        let projects_array: Vec<Project> = value
            .get("open_projects")
            .expect("open_proects field")
            .as_array()
            .expect("Valid array")
            .iter()
            .map(|v| {
                let state = serde_json::from_value::<ProjectSaveState>(v.clone()).unwrap();
                return match state {
                    ProjectSaveState::Saved(path) => {
                        match fs::read_to_string(path.clone()) {
                            Ok(s) => {
                                let project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");
                                Some(project)
                            }
                            Err(_) => {
                                warn!("Could not Load Saved Project at path {:?}", path);
                                None
                            }
                        }
                    }
                    ProjectSaveState::NotSaved(path) => {
                        match path {
                            None => {
                                warn!("Could not Load Not Saved Project");
                                None
                            }
                            Some(p) => {
                                match fs::read_to_string(p.clone()) {
                                    Ok(s) => {
                                        let project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");
                                        Some(project)
                                    }
                                    Err(_) => {
                                        warn!("Could not Load Not Saved Project at path {:?}", p);
                                        Some(Project::default())
                                    }
                                }
                            }
                        }
                    }
                };
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect();

        return Ok(EditorData {
            current_project_index: 0,
            projects: projects_array,
            history: history_array,
        });
    }
}