use std::{
    fs,
    path::{Path, PathBuf},
};

use super::error::*;

#[must_use]
pub fn write_with_mkdir<P, C>(filepath: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let filepath = filepath.as_ref();

    if let Some(dir) = filepath.parent() {
        fs::create_dir_all(dir).map_err(|e| Error {
            action: ActionKind::CreateDir,
            path: dir.to_owned(),
            source: e,
        })?
    }
    fs::write(filepath, contents).map_err(|e| Error {
        action: ActionKind::WriteFile,
        path: filepath.to_owned(),
        source: e,
    })
}

#[must_use]
pub fn read_to_string(filepath: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(&filepath).map_err(|e| Error {
        action: ActionKind::ReadFile,
        path: filepath.as_ref().to_owned(),
        source: e,
    })
}

#[must_use]
pub fn remove_file(filepath: impl AsRef<Path>) -> Result<()> {
    fs::remove_file(&filepath).map_err(|e| Error {
        action: ActionKind::RemoveFile,
        path: filepath.as_ref().to_owned(),
        source: e,
    })
}

pub struct SingleFileDriver {
    pub filepath: PathBuf,
}

impl SingleFileDriver {
    pub fn new(filepath: impl AsRef<Path>) -> Self {
        Self {
            filepath: filepath.as_ref().to_owned(),
        }
    }

    #[must_use]
    pub fn write(&self, contents: &str) -> Result<()> {
        self::write_with_mkdir(&self.filepath, contents)
    }

    #[must_use]
    pub fn read(&self) -> Result<String> {
        self::read_to_string(&self.filepath)
    }

    #[must_use]
    pub fn remove(&self) -> Result<()> {
        self::remove_file(&self.filepath)
    }
}
