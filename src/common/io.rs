use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum LoadError {
    NoAutoSave,
    DirectoryNotFound,
    ParseError,
}

#[derive(Debug)]
pub enum SaveError {
    DirectoryNotFound(String),
    InvalidPath(anyhow::Error),
}

pub trait Load<T> {
    fn load(&self) -> Result<T, LoadError>;
}

pub trait Save<T> {
    fn save(&self, value: &T) -> Result<(), SaveError>;
}

pub fn recurse_files(path: impl AsRef<Path>) -> std::io::Result<Vec<PathBuf>> {
    let mut buf = vec![];
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;

        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            buf.append(&mut subdir);
        }

        if meta.is_file() {
            buf.push(entry.path());
        }
    }

    Ok(buf)
}
