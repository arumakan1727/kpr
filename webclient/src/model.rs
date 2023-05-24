use crate::error::*;
use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug};

pub use reqwest::Url;

pub type LocalDateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumIter)]
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
    pub problems: Vec<ProblemInfo>,
    pub start_at: LocalDateTime,
    pub end_at: LocalDateTime,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProblemInfo {
    pub url: String,
    pub ord: u32,
    pub id: String,
    pub title: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[async_trait]
pub trait Client {
    fn platform(&self) -> Platform;

    fn is_contest_url(&self, url: &Url) -> bool;

    fn is_problem_url(&self, url: &Url) -> bool;

    fn get_problem_id(&self, url_path: &str) -> Option<String>;

    async fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo>;

    async fn fetch_testcases(&self, problem_url: &Url) -> Result<Vec<Testcase>>;

    fn credential_fields(&self) -> &'static [CredField];

    fn is_logged_in(&self) -> bool;

    async fn login(&mut self, cred: CredMap) -> Result<()>;

    fn export_authtoken_as_json(&self) -> String;

    fn load_authtoken_json(&mut self, serialized_auth: &str) -> Result<()>;

    async fn logout(&mut self) -> Result<()>;

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<()>;
}
