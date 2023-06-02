pub mod atom;
pub mod contents;
pub mod credential;
pub mod problem_id;

pub use atom::*;
pub use contents::*;
pub use credential::*;
pub use problem_id::ProblemId;

use crate::error::Result;
use async_trait::async_trait;

pub trait UrlAnalyzer {
    fn is_supported_origin(url: &Url) -> bool;
    fn is_contest_home_url(url: &Url) -> bool;
    fn is_problem_url(url: &Url) -> bool;
    fn extract_problem_id(url: &Url) -> problem_id::Result<ProblemId>;
}

#[async_trait]
pub trait Client {
    fn platform(&self) -> Platform;

    fn is_contest_home_url(&self, url: &Url) -> bool;

    fn is_problem_url(&self, url: &Url) -> bool;

    fn extract_problem_id(&self, url: &Url) -> problem_id::Result<ProblemId>;

    async fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo>;

    async fn fetch_problem_detail(&self, problem_url: &Url)
        -> Result<(ProblemMeta, Vec<Testcase>)>;

    fn credential_fields(&self) -> &'static [CredFieldMeta];

    fn is_logged_in(&self) -> bool;

    async fn login(&mut self, cred: CredMap) -> Result<()>;

    fn export_authtoken_as_json(&self) -> String;

    fn load_authtoken_json(&mut self, serialized_auth: &str) -> Result<()>;

    async fn logout(&mut self) -> Result<()>;

    async fn fetch_submittable_language_list(&self) -> Result<Vec<PgLang>>;

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<()>;
}
