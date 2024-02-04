use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use bevy::prelude::{Event, Resource, Vec2};
use directories::ProjectDirs;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::map::resources::MapEntity;
use crate::project::loader::{Load, LoadError};
use crate::project::saver::{ProjectSaver, Save, SaveError};

pub(crate) mod saver;
pub(crate) mod loader;

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct Project {
    pub map_entity: MapEntity,
    pub save_state: ProjectSaveState,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum ProjectSaveState {
    /// If the Project was saved somewhere
    /// Contains the path to the saved project
    Saved(PathBuf),

    /// If the project was auto saved
    /// Contains the path to an auto save file
    AutoSaved(PathBuf),

    /// If the project was not saved
    #[default] NotSaved,
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
            save_state: ProjectSaveState::NotSaved,
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
            match &project.save_state {
                ProjectSaveState::AutoSaved(val) => ProjectSaveState::AutoSaved(val.clone()),
                ProjectSaveState::Saved(val) => ProjectSaveState::Saved(val.clone()),
                ProjectSaveState::NotSaved => {
                    println!("autosaving {}", project.map_entity.name.clone());
                    let project_saver = ProjectSaver { directory: Box::from(data_dir) };
                    project_saver.save(project).unwrap();
                    ProjectSaveState::AutoSaved(data_dir.join(format!("auto_save_{}.map", project.map_entity.name)))
                }
            }
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
            .expect("open_projects field")
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
                    ProjectSaveState::AutoSaved(path) => {
                        match fs::read_to_string(path.clone()) {
                            Ok(s) => {
                                let project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");
                                Some(project)
                            }
                            Err(_) => {
                                warn!("Could not Load Not Saved Project at path {:?}", path);
                                Some(Project::default())
                            }
                        }
                    }
                    ProjectSaveState::NotSaved => {
                        warn!("Could not open Project because it was not saved");
                        return None;
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