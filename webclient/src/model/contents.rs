use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::atom::*;
use super::problem_id::ProblemId;

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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ProblemMeta {
    pub platform: Platform,
    pub url: String,
    pub problem_id: ProblemId,
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

/// Submission language candidate
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PgLang {
    /// e.g. "C++ (GCC 9.2.1)"
    pub name: String,
    /// e.g. "4003"
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