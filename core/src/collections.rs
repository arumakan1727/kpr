pub use self::glob_map::GlobMap;

pub mod glob_map {
    use crate::serdable::GlobPattern;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GlobMap<V> {
        pairs: Vec<(GlobPattern, V)>,
    }

    pub type Iter<'a, V> = std::slice::Iter<'a, (GlobPattern, V)>;

    pub type IterMut<'a, V> = std::slice::IterMut<'a, (GlobPattern, V)>;

    impl<V> Default for GlobMap<V> {
        fn default() -> Self {
            Self { pairs: Vec::new() }
        }
    }

    impl<V> GlobMap<V> {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn is_empty(&self) -> bool {
            self.pairs.is_empty()
        }

        pub fn len(&self) -> usize {
            self.pairs.len()
        }

        pub fn reserve(&mut self, additional: usize) {
            self.pairs.reserve(additional)
        }

        pub fn shrink_to_fit(&mut self) {
            self.pairs.shrink_to_fit()
        }

        pub fn insert(&mut self, k: GlobPattern, mut v: V) -> Option<V> {
            if let Some((_, value)) = self.pairs.iter_mut().find(|(pat, _)| pat == &k) {
                std::mem::swap(value, &mut v);
                Some(v)
            } else {
                self.pairs.push((k, v));
                None
            }
        }

        pub fn get(&self, k: impl AsRef<str>) -> Option<&V> {
            let k = k.as_ref();
            self.pairs
                .iter()
                .find(|(pattern, _)| pattern.matches(k))
                .map(|(_, value)| value)
        }

        pub fn iter(&self) -> Iter<'_, V> {
            self.pairs.iter()
        }

        pub fn iter_mut(&mut self) -> IterMut<'_, V> {
            self.pairs.iter_mut()
        }
    }

    impl<V> FromIterator<(GlobPattern, V)> for GlobMap<V> {
        fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = (GlobPattern, V)>,
        {
            let mut m = Self::new();
            for (k, v) in iter {
                m.insert(k, v);
            }
            m
        }
    }

    impl<V> IntoIterator for GlobMap<V> {
        type Item = (GlobPattern, V);

        type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.pairs.into_iter()
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        const PAT_MAIN_CPP: &str = "[mM]ain.cpp";
        const PAT_CPP: &str = "*.cpp";

        #[test]
        fn just_insert_and_get() {
            let mut m = GlobMap::new();
            m.insert(GlobPattern::parse(PAT_MAIN_CPP).unwrap(), 1);
            assert_eq!(m.get("main.cpp"), Some(&1));
            assert_eq!(m.get("Main.cpp"), Some(&1));
            assert_eq!(m.get("ain.cpp"), None);
        }

        #[test]
        fn should_preserve_insertion_order() {
            {
                let mut m = GlobMap::new();
                m.insert(GlobPattern::parse(PAT_MAIN_CPP).unwrap(), 1234);
                m.insert(GlobPattern::parse(PAT_CPP).unwrap(), 777);
                assert_eq!(m.len(), 2);
                assert_eq!(m.get("main.cpp"), Some(&1234));
            }
            {
                let mut m = GlobMap::new();
                m.insert(GlobPattern::parse(PAT_CPP).unwrap(), 777);
                m.insert(GlobPattern::parse(PAT_MAIN_CPP).unwrap(), 1234);
                assert_eq!(m.len(), 2);
                assert_eq!(m.get("main.cpp"), Some(&777));
            }
        }

        #[test]
        fn calling_insert_replaces_existing_key() {
            let mut m = GlobMap::new();
            let pat = GlobPattern::parse(PAT_CPP).unwrap();

            assert_eq!(m.insert(pat.clone(), 1234), None);
            assert_eq!(m.insert(pat.clone(), 777), Some(1234));

            assert_eq!(m.len(), 1);
            assert_eq!(m.get("main.cpp"), Some(&777));
        }

        #[test]
        fn from_iter_and_into_iter() {
            let pat1 = GlobPattern::parse(PAT_MAIN_CPP).unwrap();
            let pat2 = GlobPattern::parse(PAT_CPP).unwrap();

            let m: GlobMap<_> = [(pat1.clone(), 11), (pat2.clone(), 22), (pat1.clone(), 9999)]
                .into_iter()
                .collect();
            assert_eq!(m.len(), 2);

            let vec: Vec<_> = m.into_iter().collect();
            assert_eq!(vec, [(pat1.clone(), 9999), (pat2.clone(), 22)]);
        }
    }
}
