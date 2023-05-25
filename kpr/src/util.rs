use std::{collections::HashSet, hash::Hash, path::PathBuf, process::exit};

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
        eprintln!("Cannot to get current dir: {}", e);
        exit(1);
    })
}
