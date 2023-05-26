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
    fs::create_dir_all(dir).map_err(|e| Error::SingleIO("Cannot create dir", dir.to_owned(), e))
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
    fs::write(filepath, contents)
        .map_err(|e| Error::SingleIO("Cannot write file", filepath.to_owned(), e))
}

#[must_use]
pub fn read_to_string(filepath: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(&filepath)
        .map_err(|e| Error::SingleIO("Cannot read file", filepath.as_ref().to_owned(), e))
}

#[must_use]
pub fn remove_file(filepath: impl AsRef<Path>) -> Result<()> {
    fs::remove_file(&filepath)
        .map_err(|e| Error::SingleIO("Cannot remove file", filepath.as_ref().to_owned(), e))
}

#[must_use]
pub fn write_json_with_mkdir<P, T>(filepath: P, data: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let s = serde_json::to_string(data)
        .map_err(|e| Error::SerializeToJson(filepath.as_ref().to_owned(), e))?;
    write_with_mkdir(filepath, &s)
}

#[must_use]
pub fn read_json_with_deserialize<P, T>(filepath: P) -> Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let filepath = filepath.as_ref();
    let f = File::open(&filepath)
        .map_err(|e| Error::SingleIO("Cannot read file", filepath.to_owned(), e))?;
    serde_json::from_reader(BufReader::new(f))
        .map_err(|e| Error::DeserializeFromJson(filepath.to_owned(), e))
}

#[must_use]
#[cfg(unix)]
pub fn symlink_file_with_mkdir(orig: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    if let Some(dir) = link.as_ref().parent() {
        self::mkdir_all(dir)?;
    }
    use std::os::unix;
    unix::fs::symlink(&orig, &link)
        .map_err(|e| Error::Symlink(orig.as_ref().to_owned(), link.as_ref().to_owned(), e))
}

#[must_use]
#[cfg(windows)]
pub fn symlink_file_with_mkdir(orig: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    use std::os::windows;
    windows::fs::symlink_file(&orig, &link)
        .map_err(|e| Error::Symlink(orig.as_ref().to_owned(), link.as_ref().to_owned(), e))
}

#[must_use]
#[cfg(unix)]
pub fn symlink_dir_with_mkdir(orig: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    // On unix, it is not necessary to distinguish whether the symlink origin is a file or a directory.
    self::symlink_file_with_mkdir(orig, link)
}

#[must_use]
#[cfg(windows)]
pub fn symlink_dir_with_mkdir(orig: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    use std::os::windows;
    windows::fs::symlink_dir(&orig, &link)
        .map_err(|e| Error::Symlink(orig.as_ref().to_owned(), link.as_ref().to_owned(), e))
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
