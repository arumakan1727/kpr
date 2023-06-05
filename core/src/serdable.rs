pub use self::glob::GlobPattern;

pub mod glob {
    use std::ops::{Deref, DerefMut};

    use ::glob::PatternError;
    use ::serde::{
        de::{self, Visitor},
        Deserialize, Serialize,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GlobPattern(::glob::Pattern);

    impl GlobPattern {
        pub fn parse(pattern: &str) -> Result<Self, PatternError> {
            ::glob::Pattern::new(pattern).map(Self)
        }
    }

    impl Deref for GlobPattern {
        type Target = ::glob::Pattern;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for GlobPattern {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl Serialize for GlobPattern {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(self.0.as_str())
        }
    }

    impl<'de> Deserialize<'de> for GlobPattern {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct GlobPatternVisitor;

            impl<'de> Visitor<'de> for GlobPatternVisitor {
                type Value = GlobPattern;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "a glob pattern string")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Self::Value::parse(v).map_err(de::Error::custom)
                }
            }

            deserializer.deserialize_str(GlobPatternVisitor)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        const PATTERN: &str = "*.[hc]pp";
        const SERIALIZED_PATTERN: &str = concat!('"', "*.[hc]pp", '"');

        #[test]
        fn serialize_glob_pattern_ok() {
            let pat = GlobPattern::parse(PATTERN).unwrap();
            let json = serde_json::to_string(&pat).unwrap();
            assert_eq!(json, SERIALIZED_PATTERN);
        }

        #[test]
        fn deserialize_glob_pattern_ok() {
            let pat: GlobPattern = serde_json::from_str(SERIALIZED_PATTERN).unwrap();
            assert_eq!(pat.as_str(), PATTERN);
        }

        #[test]
        fn deserialize_glob_pattern_ng() {
            let res: Result<GlobPattern, _> = serde_json::from_str("[a");
            assert!(res.is_err());
            dbg!(res.unwrap_err());
        }
    }
}
