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