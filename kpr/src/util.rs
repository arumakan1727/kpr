use std::{collections::HashSet, hash::Hash};

pub fn dedup<T>(mut v: Vec<T>) -> Vec<T>
where
    T: Hash + Eq + Copy,
{
    let mut set = HashSet::new();
    v.retain(|&x| set.insert(x));
    v
}
