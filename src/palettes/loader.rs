use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use log::{info, warn};
use serde_json::Value;

use crate::common::io::{Load, LoadError, recurse_files};
use crate::palettes::Palette;

pub struct PalettesLoader {
    pub parent_dir: PathBuf,
}

impl PalettesLoader {
    pub fn new(dir_path: PathBuf) -> Self {
        return Self {
            parent_dir: dir_path
        };
    }
}

impl Load<HashMap<String, Palette>> for PalettesLoader {
    fn load(&self) -> Result<HashMap<String, Palette>, LoadError> {
        let files = recurse_files(&self.parent_dir).unwrap();
        let mut palettes = HashMap::new();

        for path in files.iter() {
            match serde_json::from_str::<Vec<Value>>(&fs::read_to_string(path).unwrap()) {
                Err(_) => {
                    warn!("Failed to deserialize {:?} to Vec of Values", path);
                    continue;
                }
                Ok(p) => {
                    for element in p {
                        match serde_json::from_value::<Palette>(element) {
                            Err(e) => {
                                warn!("Failed to deserialize object in file {:?} as palette {:?}", path, e);
                                continue;
                            }
                            Ok(p) => {
                                info!("Successfully serialized Palette {} in {:?}", p.id, path);
                                palettes.insert(p.id.clone(), p)
                            }
                        };
                    }
                }
            };
        }

        return Ok(palettes);
    }
}