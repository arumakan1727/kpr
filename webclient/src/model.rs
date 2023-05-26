use crate::{atcoder::AtCoderUrlAnalyzer, error::*};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, time::Duration};

pub use reqwest::Url;

pub type LocalDateTime = chrono::DateTime<chrono::Local>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumIter, Serialize, Deserialize,
)]
pub enum Platform {
    AtCoder,
}

impl Platform {
    pub const fn lowercase(&self) -> &'static str {
        use Platform::*;
        match self {
            AtCoder => "atcoder",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContestInfo {
    pub url: String,
    pub short_title: String,
    pub long_title: String,
    pub problems: Vec<ContestProblemOutline>,
    pub start_at: LocalDateTime,
    pub end_at: LocalDateTime,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContestProblemOutline {
    pub url: String,
    pub ord: u32,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
pub struct IdName(pub(crate) String);

pub type IdNameResult = std::result::Result<IdName, IdNameError>;

#[derive(Debug, thiserror::Error)]
pub enum IdNameError {
    #[error("Cannot parse as URL (given '{0}')")]
    CannotParseAsUrl(String),

    #[error("Unknown URL origin for kpr-platform: '{0}'")]
    UnknownOrigin(Url),

    #[error("Not a problem URL of {1}: '{0}'")]
    NotProblemUrl(Url, Platform),
}

impl<'a> TryFrom<&'a Url> for IdName {
    type Error = IdNameError;

    fn try_from(url: &'a Url) -> IdNameResult {
        let Some(platform) = crate::detect_platform_from_url(url) else {
            return Err(IdNameError::UnknownOrigin(url.to_owned()));
        };
        use Platform::*;
        match platform {
            AtCoder => AtCoderUrlAnalyzer::problem_id_name(url),
        }
    }
}

impl std::fmt::Display for IdName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for IdName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ProblemMeta {
    pub platform: Platform,
    pub url: String,
    pub id_name: IdName,
    pub title: String,
    pub execution_time_limit: Duration,
    pub memory_limit_kb: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Testcase {
    pub ord: u32,
    pub input: String,
    pub expected: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PgLang {
    pub name: String,
    pub id: String,
}

impl PgLang {
    pub fn new<S1, S2>(name: S1, id: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            name: name.into(),
            id: id.into(),
        }
    }
}

pub type CredName = &'static str;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CredFieldKind {
    Text,
    Password,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CredField {
    pub name: CredName,
    pub kind: CredFieldKind,
}

/// Credential table.
/// e.g. `[ "username" => "Bob", "password" => "***" ]`
pub type CredMap = HashMap<CredName, String>;

pub trait UrlAnalyzer {
    fn is_supported_origin(url: &Url) -> bool;
    fn is_contest_home_url(url: &Url) -> bool;
    fn is_problem_url(url: &Url) -> bool;
    fn problem_id_name(url: &Url) -> std::result::Result<IdName, IdNameError>;
}

#[async_trait]
pub trait Client {
    fn platform(&self) -> Platform;

    fn is_contest_home_url(&self, url: &Url) -> bool;

    fn is_problem_url(&self, url: &Url) -> bool;

    fn problem_id_name(&self, url: &Url) -> IdNameResult;

    async fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo>;

    async fn fetch_problem_detail(&self, problem_url: &Url)
        -> Result<(ProblemMeta, Vec<Testcase>)>;

    fn credential_fields(&self) -> &'static [CredField];

    fn is_logged_in(&self) -> bool;

    async fn login(&mut self, cred: CredMap) -> Result<()>;

    fn export_authtoken_as_json(&self) -> String;

    fn load_authtoken_json(&mut self, serialized_auth: &str) -> Result<()>;

    async fn logout(&mut self) -> Result<()>;

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<()>;
}
