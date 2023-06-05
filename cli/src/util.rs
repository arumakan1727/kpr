use std::{
    collections::HashSet,
    hash::Hash,
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::{bail, Context};
use kpr_core::{fsutil, serdable::GlobPattern};

pub fn dedup<T>(mut v: Vec<T>) -> Vec<T>
where
    T: Hash + Eq + Copy,
{
    let mut set = HashSet::new();
    v.retain(|&x| set.insert(x));
    v
}

pub fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|e| {
        log::error!("Failed to get current dir: {}", e);
        exit(1);
    })
}

pub fn replace_homedir_to_tilde(path: impl Into<PathBuf>) -> PathBuf {
    let path = path.into();
    let Some(home_dir) = ::dirs::home_dir() else {
        return path
    };
    path.strip_prefix(home_dir)
        .map(|path| Path::new("~").join(path))
        .unwrap_or(path)
}

pub fn determine_program_file(
    program_file_or_dir: &Option<PathBuf>,
    file_pattern: &GlobPattern,
) -> anyhow::Result<PathBuf> {
    let existing_path = match program_file_or_dir {
        Some(path) if path.exists() => path,
        Some(path) => bail!("No such file or dir: {:?}", path),
        None => Path::new("./"),
    };

    if existing_path.is_dir() {
        fsutil::find_most_recently_modified_file(&existing_path, &file_pattern)
            .with_context(|| format!("Cannot find target program file in {:?}", existing_path))
    } else {
        Ok(existing_path.to_owned())
    }
}
