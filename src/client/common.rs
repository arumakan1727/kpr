use std::collections::HashMap;

use crate::errors::Result;
use async_trait::async_trait;
use chrono::DateTime;

pub use reqwest::Url;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContestInfo {
    pub url: String,
    pub short_title: String,
    pub long_title: String,
    pub problems: Vec<ProblemInfo>,
    pub start_at: DateTime<chrono::Local>,
    pub end_at: DateTime<chrono::Local>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProblemInfo {
    pub url: String,
    pub ord: u32,
    pub short_title: String,
    pub long_title: String,
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
    pub fn new(name: &str, id: &str) -> Self {
        Self {
            name: name.to_owned(),
            id: id.to_owned(),
        }
    }
}

pub type CredMap<'a> = HashMap<&'static str, &'a str>;

pub trait IntoCredMap: Send {
    fn into_cred_map(&self) -> CredMap;
}

#[async_trait]
pub trait Client {
    fn is_contest_url(&self, url: &Url) -> bool;

    fn is_problem_url(&self, url: &Url) -> bool;

    async fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo>;

    async fn fetch_testcases(&self, problem_url: &Url) -> Result<Vec<Testcase>>;

    async fn login(&mut self, cred: Box<dyn IntoCredMap>) -> Result<()>;

    fn ask_credential(&self) -> Result<Box<dyn IntoCredMap>>;

    async fn logout(&mut self) -> Result<()>;

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<()>;
}
