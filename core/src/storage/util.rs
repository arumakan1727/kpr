use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use super::error::*;

#[must_use]
pub fn mkdir_all(path: impl AsRef<Path>) -> Result<()> {
    let dir = path.as_ref();
    fs::create_dir_all(dir).map_err(|e| Error {
        action: ActionKind::CreateDir,
        path: dir.to_owned(),
        source: Box::from(e),
    })
}

#[must_use]
pub fn write_with_mkdir<P, C>(filepath: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let filepath = filepath.as_ref();

    if let Some(dir) = filepath.parent() {
        self::mkdir_all(dir)?;
    }
    fs::write(filepath, contents).map_err(|e| Error {
        action: ActionKind::WriteFile,
        path: filepath.to_owned(),
        source: Box::from(e),
    })
}

#[must_use]
pub fn read_to_string(filepath: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(&filepath).map_err(|e| Error {
        action: ActionKind::ReadFile,
        path: filepath.as_ref().to_owned(),
        source: Box::from(e),
    })
}

#[must_use]
pub fn remove_file(filepath: impl AsRef<Path>) -> Result<()> {
    fs::remove_file(&filepath).map_err(|e| Error {
        action: ActionKind::RemoveFile,
        path: filepath.as_ref().to_owned(),
        source: Box::from(e),
    })
}

#[must_use]
pub fn write_json_with_mkdir<P, T>(filepath: P, data: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let s = serde_json::to_string(data).map_err(|e| Error {
        action: ActionKind::SerializeToJson,
        path: filepath.as_ref().to_owned(),
        source: Box::from(e),
    })?;

    write_with_mkdir(filepath, &s)
}

#[must_use]
pub fn read_json_with_deserialize<P, T>(filepath: P) -> Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let f = File::open(&filepath).map_err(|e| Error {
        action: ActionKind::ReadFile,
        path: filepath.as_ref().to_owned(),
        source: Box::from(e),
    })?;
    let r = BufReader::new(f);
    serde_json::from_reader(r).map_err(|e| Error {
        action: ActionKind::DeserializeFromJson,
        path: filepath.as_ref().to_owned(),
        source: Box::from(e),
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
