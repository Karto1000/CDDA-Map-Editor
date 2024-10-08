use std::fs;
use std::fs::File;
use std::io::Write;

use bevy::log::info;
use bevy::prelude::default;
use directories::ProjectDirs;
use serde_json::{Map, Value};

use crate::common::io::{Load, LoadError, Save, SaveError};
use crate::common::io::LoadError::ParseError;
use crate::program::data::{Menus, Program, ProgramState};
use crate::project::data::{Project, ProjectSaveState};
use crate::project::io::ProjectSaver;

pub struct ProgramdataLoader {}

impl Load<Program> for ProgramdataLoader {
    fn load(&self) -> Result<Program, LoadError> {
        let dir = match ProjectDirs::from_path("CDDA Map Editor".into()) {
            None => { return Err(LoadError::DirectoryNotFound); }
            Some(d) => d
        };

        let data_dir = dir.data_local_dir();

        if !data_dir.exists() { fs::create_dir_all(data_dir).unwrap(); }

        let contents = match fs::read_to_string(data_dir.join("data.json")) {
            Err(_) => return Err(ParseError),
            Ok(f) => f
        };

        let value: Map<String, Value> = serde_json::from_str(contents.as_str())
            .expect("Valid Json");

        // let history_array: Vec<ProjectSaveState> = value
        //     .get("history")
        //     .expect("history Field")
        //     .as_array()
        //     .expect("Valid Array")
        //     .iter()
        //     .map(|v| serde_json::from_value::<ProjectSaveState>(v.clone()).unwrap())
        //     .collect();

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

                                info!("Loaded Saved Project at Path {:?}", path);

                                Some(project)
                            }
                            Err(_) => {
                                log::warn!("Could not Load Saved Project at path {:?}", path);
                                None
                            }
                        }
                    }
                    ProjectSaveState::AutoSaved(path) => {
                        match fs::read_to_string(path.clone()) {
                            Ok(s) => {
                                let project: Project = serde_json::from_str(s.as_str()).expect("Valid Project");

                                info!("Loaded Auto saved Project at Path {:?}", path);

                                Some(project)
                            }
                            Err(_) => {
                                log::warn!("Could not Load Not Saved Project at path {:?}", path);
                                Some(Project::default())
                            }
                        }
                    }
                    ProjectSaveState::NotSaved => {
                        log::warn!("Could not open Project because it was not saved");
                        return None;
                    }
                };
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .collect();

        return Ok(Program::new(projects_array, vec![]));
    }
}

pub struct ProgramdataSaver {}

impl Save<Program> for ProgramdataSaver {
    fn save(&self, value: &Program) -> Result<(), SaveError> {
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

        let mut data: Map<String, Value> = Map::new();

        let open_projects: Vec<ProjectSaveState> = value.projects.iter().map(|project| {
            match &project.save_state {
                ProjectSaveState::AutoSaved(val) => ProjectSaveState::AutoSaved(val.clone()),
                ProjectSaveState::Saved(val) => ProjectSaveState::Saved(val.clone()),
                ProjectSaveState::NotSaved => {
                    info!("autosaving {}", project.name);
                    let project_saver = ProjectSaver { directory: Box::from(data_dir) };
                    project_saver.save(project).unwrap();
                    ProjectSaveState::AutoSaved(data_dir.join(format!("auto_save_{}.map", project.name)))
                }
            }
        }).collect();

        data.insert("open_projects".into(), serde_json::to_value(open_projects).unwrap());

        file.write_all(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();

        return Ok(());
    }
}