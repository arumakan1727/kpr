use std::{collections::HashMap, fmt::Debug};

use crate::errors::Result;
use async_trait::async_trait;
use chrono::DateTime;
use thiserror::Error;

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

pub trait IntoCredMap: Debug + Send {
    fn into_cred_map(&self) -> CredMap;
}

pub trait JsonableAuth {
    fn to_json(&self) -> String;
}

#[async_trait]
pub trait Client {
    fn platform_name(&self) -> &'static str;

    fn is_contest_url(&self, url: &Url) -> bool;

    fn is_problem_url(&self, url: &Url) -> bool;

    async fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo>;

    async fn fetch_testcases(&self, problem_url: &Url) -> Result<Vec<Testcase>>;

    async fn login(&mut self, cred: Box<dyn IntoCredMap>) -> Result<()>;

    fn auth_data(&self) -> Box<dyn JsonableAuth>;

    fn ask_credential(&self) -> Result<Box<dyn IntoCredMap>>;

    async fn logout(&mut self) -> Result<()>;

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<()>;
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Wrong {fields}")]
    WrongCredential { fields: &'static str },

    #[error("Need login")]
    NeedLogin,
}
