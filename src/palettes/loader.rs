use std::fs;
use std::path::PathBuf;

use crate::palettes::Palette;
use crate::project::loader::{Load, LoadError};

pub struct PaletteLoader {
    pub path: PathBuf,
}

impl Load<Vec<Palette>> for PaletteLoader {
    fn load(&self) -> Result<Vec<Palette>, LoadError> {
        return Ok(serde_json::from_str(fs::read_to_string(self.path.clone()).unwrap().as_str()).unwrap());
    }
}