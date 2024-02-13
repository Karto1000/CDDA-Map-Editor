use std::fs;
use std::path::PathBuf;
use crate::common::io::{Load, LoadError};

use crate::palettes::Palette;

pub struct PaletteLoader {
    pub path: PathBuf,
}

impl PaletteLoader {
    pub fn new(path: PathBuf) -> Self {
        return Self {
            path
        };
    }
}

impl Load<Vec<Palette>> for PaletteLoader {
    fn load(&self) -> Result<Vec<Palette>, LoadError> {
        return Ok(serde_json::from_str(fs::read_to_string(self.path.clone()).unwrap().as_str()).unwrap());
    }
}