use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{self, File, ReadDir},
    io::BufReader,
    path::{Path, PathBuf},
};

pub mod error {
    use std::{io, path::PathBuf};

    pub type Result<T> = std::result::Result<T, self::Error>;

    type Msg = &'static str;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("{0} ({1}): {2}")]
        SingleIO(Msg, PathBuf, #[source] io::Error),

        #[error("{0} (from='{1}', to='{2}): {3}")]
        FromToIO(Msg, PathBuf, PathBuf, #[source] io::Error),

        #[error("Cannot create symlink (orig='{0}', link={1}): {2}")]
        Symlink(PathBuf, PathBuf, #[source] io::Error),

        #[error("Failed to canonicalize path '{0}': {1}")]
        CanonicalizePath(PathBuf, #[source] io::Error),

        #[error("Cannot serialize to JSON (dest='{0}'): {1}")]
        SerializeToJson(PathBuf, #[source] serde_json::Error),

        #[error("Cannot deserialize from JSON (src='{0}'): {1}")]
        DeserializeFromJson(PathBuf, #[source] serde_json::Error),
    }
}
use error::*;

#[must_use]
pub fn mkdir_all(path: impl AsRef<Path>) -> Result<()> {
    let dir = path.as_ref();
    fs::create_dir_all(dir).map_err(|e| Error::SingleIO("Cannot create dir", dir.to_owned(), e))
}

#[must_use]
pub fn write<P, C>(filepath: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    fs::write(&filepath, contents)
        .map_err(|e| Error::SingleIO("Cannot write file", filepath.as_ref().to_owned(), e))
}

#[must_use]
pub fn write_with_mkdir<P, C>(filepath: P, contents: C) -> Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    if let Some(dir) = filepath.as_ref().parent() {
        self::mkdir_all(dir)?;
    }
    self::write(filepath, contents)
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
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<u64> {
    fs::copy(&from, &to).map_err(|e| {
        Error::FromToIO(
            "Cannot copy file",
            from.as_ref().to_owned(),
            to.as_ref().to_owned(),
            e,
        )
    })
}

#[must_use]
pub fn copy_contents_all(src_dir: impl AsRef<Path>, dst_dir: impl AsRef<Path>) -> Result<()> {
    self::mkdir_all(&dst_dir)?;
    for entry in self::read_dir(&src_dir)? {
        let entry = entry.map_err(|e| {
            Error::FromToIO(
                "Cannot access dir entry on `copy_contents_all()`",
                src_dir.as_ref().to_owned(),
                dst_dir.as_ref().to_owned(),
                e,
            )
        })?;
        let dst = dst_dir.as_ref().join(entry.file_name());
        let ty = entry.file_type().map_err(|e| {
            Error::SingleIO(
                "Cannot get filetype on `copy_contents_all()`",
                entry.path(),
                e,
            )
        })?;
        if ty.is_dir() {
            self::copy_contents_all(entry.path(), dst)?;
        } else {
            self::copy_file(entry.path(), dst)?;
        }
    }
    Ok(())
}

#[must_use]
pub fn read_dir(dir: impl AsRef<Path>) -> Result<ReadDir> {
    fs::read_dir(&dir).map_err(|e| Error::SingleIO("Cannot read dir", dir.as_ref().to_owned(), e))
}

#[must_use]
#[cfg(unix)]
pub fn symlink(orig: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    let link = link.as_ref();
    if link.is_symlink() {
        fs::remove_file(&link).map_err(|e| {
            Error::SingleIO(
                "Cannot create symlink: failed to remove existing symlink",
                link.to_owned(),
                e,
            )
        })?;
    }
    use std::os::unix;
    unix::fs::symlink(&orig, &link)
        .map_err(|e| Error::Symlink(orig.as_ref().to_owned(), link.to_owned(), e))
}

#[must_use]
pub fn symlink_with_mkdir(orig: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    if let Some(dir) = link.as_ref().parent() {
        self::mkdir_all(dir)?;
    }
    self::symlink(orig, link)
}

pub fn symlink_using_relpath_with_mkdir(
    orig: impl AsRef<Path>,
    link: impl AsRef<Path>,
) -> Result<()> {
    if let Some(dir) = link.as_ref().parent() {
        self::mkdir_all(dir)?;
    }
    let relpath = self::relative_path(&link, &orig)?;
    symlink(relpath, link)
}

pub fn canonicalize_path(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    path.canonicalize()
        .map_err(|e| Error::CanonicalizePath(path.to_owned(), e))
}

/// Calc relative path.
/// ```
/// use kpr_core::fsutil::relative_path;
/// use std::path::Path;
///
/// let res = relative_path("/usr/bin/curl", "/usr/share/").unwrap();
/// assert_eq!(res, Path::new("../share"));
///
/// let res = relative_path("/usr/bin/curl", "/usr").unwrap();
/// assert_eq!(res, Path::new(".."));
///
/// let res = relative_path("/usr", "/usr/bin/curl").unwrap();
/// assert_eq!(res, Path::new("bin/curl"));
///
/// let res = relative_path("/usr/bin/curl", "/usr/bin").unwrap();
/// assert_eq!(res, Path::new("."));
///
/// // When `from` is a directory
/// let res = relative_path("/bin", "/bin").unwrap();
/// assert_eq!(res, Path::new("."));
///
/// // When `from` is a file
/// let res = relative_path("/bin/sh", "/bin/sh").unwrap();
/// assert_eq!(res, Path::new("sh"));
/// ```
pub fn relative_path<P1, P2>(from: P1, to: P2) -> Result<PathBuf>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let from = from.as_ref();
    let from_dir = if from.is_dir() {
        from
    } else {
        from.parent().unwrap()
    };

    let from_dir = self::canonicalize_path(from_dir)?;
    let to = self::canonicalize_path(to)?;

    if from_dir == to {
        return Ok(PathBuf::from("."));
    }

    let mut ans = PathBuf::new();
    let mut dir = from_dir.as_path();
    while !to.starts_with(dir) {
        dir = dir.parent().unwrap();
        ans.push("..");
    }
    ans.push(to.strip_prefix(dir).unwrap());
    Ok(ans)
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
