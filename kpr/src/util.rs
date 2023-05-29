use std::{
    collections::HashSet,
    hash::Hash,
    path::{Path, PathBuf},
    process::exit,
};

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
        eprintln!("Failed to get current dir: {}", e);
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
