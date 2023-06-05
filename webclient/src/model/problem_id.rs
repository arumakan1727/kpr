use std::path::Path;

use super::atom::{Platform, Url};
use crate::{util, AtCoderUrlAnalyzer, UrlAnalyzer as _};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Cannot parse as URL (given '{0}')")]
    CannotParseAsUrl(String),

    #[error("Unknown URL origin for kpr-platform: '{0}'")]
    UnknownOrigin(Url),

    #[error("Not a problem URL of {1}: '{0}'")]
    NotProblemUrl(Url, Platform),
}

/// Problem identification.
/// (e.g.) "abc234_a", "typical90_az", "practice2_a", "1234"
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct ProblemId(pub(crate) String);

impl ProblemId {
    #[allow(dead_code)]
    pub(crate) fn new(problem_id: impl AsRef<str>) -> Self {
        Self(problem_id.as_ref().to_string())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<'a> TryFrom<&'a Url> for ProblemId {
    type Error = Error;

    fn try_from(url: &'a Url) -> Result<Self> {
        let Some(platform) = crate::detect_platform_from_url(url) else {
            return Err(Error::UnknownOrigin(url.to_owned()));
        };
        use Platform::*;
        match platform {
            AtCoder => AtCoderUrlAnalyzer::extract_problem_id(url),
        }
    }
}

impl std::fmt::Display for ProblemId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ProblemId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<Path> for ProblemId {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl From<ProblemId> for String {
    fn from(value: ProblemId) -> Self {
        value.0
    }
}

/// Global problem identification.
/// (e.g.) "abc234_a", "atcoder_typical90_az",
///        "atcoder_practice2_a", "aoj1234"
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ProblemGlobalId {
    pub(crate) platform: Platform,
    pub(crate) problem_id: ProblemId,
}

impl ProblemGlobalId {
    pub fn new(platform: Platform, problem_id: ProblemId) -> Self {
        Self {
            platform,
            problem_id,
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn problem_id(&self) -> &ProblemId {
        &self.problem_id
    }
}

impl std::fmt::Display for ProblemGlobalId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let id = self.problem_id.as_str();
        if self.platform == Platform::AtCoder
            && util::starts_with_anyone(id, &["abc", "arc", "agc"])
        {
            return write!(f, "{}", id);
        }
        write!(f, "{}_{}", self.platform.lowercase(), id)
    }
}

impl AsRef<ProblemId> for ProblemGlobalId {
    fn as_ref(&self) -> &ProblemId {
        &self.problem_id
    }
}

impl From<ProblemGlobalId> for Platform {
    fn from(value: ProblemGlobalId) -> Self {
        value.platform
    }
}

impl From<ProblemGlobalId> for String {
    fn from(value: ProblemGlobalId) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn atcoder_problem_global_id() {
        use Platform::*;
        assert_eq!(
            ProblemGlobalId::new(AtCoder, ProblemId::new("abc234_a")).to_string(),
            "abc234_a"
        );
        assert_eq!(
            ProblemGlobalId::new(AtCoder, ProblemId::new("arc001_1")).to_string(),
            "arc001_1"
        );
        assert_eq!(
            ProblemGlobalId::new(AtCoder, ProblemId::new("agc001_4")).to_string(),
            "agc001_4"
        );
        assert_eq!(
            ProblemGlobalId::new(AtCoder, ProblemId::new("practice2_a")).to_string(),
            "atcoder_practice2_a"
        );
        assert_eq!(
            ProblemGlobalId::new(AtCoder, ProblemId::new("typical90_az")).to_string(),
            "atcoder_typical90_az"
        );
    }
}
