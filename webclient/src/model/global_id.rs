use super::atom::{Platform, Url};
use crate::{AtCoderUrlAnalyzer, UrlAnalyzer as _};
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot parse as URL (given '{0}')")]
    CannotParseAsUrl(String),

    #[error("Unknown URL origin for kpr-platform: '{0}'")]
    UnknownOrigin(Url),

    #[error("Not a problem URL of {1}: '{0}'")]
    NotProblemUrl(Url, Platform),
}

/// Globally unique problem identification.
/// (e.g.) "abc234_a", "atcoder_typical90_az", "aoj1234"
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct GlobalId(pub(crate) String);

impl<'a> TryFrom<&'a Url> for GlobalId {
    type Error = Error;

    fn try_from(url: &'a Url) -> Result<Self> {
        let Some(platform) = crate::detect_platform_from_url(url) else {
            return Err(Error::UnknownOrigin(url.to_owned()));
        };
        use Platform::*;
        match platform {
            AtCoder => AtCoderUrlAnalyzer::problem_global_id(url),
        }
    }
}

impl std::fmt::Display for GlobalId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for GlobalId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
