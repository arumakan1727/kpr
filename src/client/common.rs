use crate::errors::Result;
use chrono::DateTime;

pub use reqwest::Url;

pub struct ContestInfo {
    pub url: String,
    pub short_title: String,
    pub long_title: String,
    pub problems: Vec<ProblemInfo>,
    pub start_at: DateTime<chrono::Local>,
    pub end_at: DateTime<chrono::Local>,
}

pub struct ProblemInfo {
    /// e.g.) "https://atcoder.jp/contests/abc095/tasks/arc096_b"
    pub url: String,

    /// First problem is 1, second problem is 2, ...
    pub ord: u32,

    // e.g.) A
    pub short_title: String,

    // e.g.) Stathic_Sushi
    pub long_title: String,
}

pub struct Testcase {
    pub ord: u32,
    pub input: String,
    pub expected: String,
}

pub struct PgLang {
    pub name: String,
    pub id: String,
}

pub type SubmissionID = u64;

pub trait Client {
    type Credential;

    fn is_contest_url(&self, url: &Url) -> bool;

    fn is_problem_url(&self, url: &Url) -> bool;

    fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo>;

    fn fetch_problem_info(&self, problem_url: &Url) -> Result<ProblemInfo>;

    fn fetch_testcases(&self, problem_url: &Url) -> Result<Vec<Testcase>>;

    fn login(&mut self, cred: &Self::Credential) -> Result<()>;

    fn ask_credential(&self) -> Result<&Self::Credential>;

    fn logout(&mut self) -> Result<()>;

    fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<SubmissionID>;
}
