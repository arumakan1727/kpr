use super::common::*;
use crate::errors::*;
use lazy_regex::{lazy_regex, Lazy, Regex};

pub struct AtCoderClient {}

pub struct Cred {
    email: String,
    password: String,
}

static RE_CONTEST_URL_PATH: Lazy<Regex> = lazy_regex!(r"^/contests/([[:alnum:]]+)/?$");

static RE_PROBLEM_URL_PATH: Lazy<Regex> =
    lazy_regex!(r"^/contests/([[:alnum:]]+)/tasks/([[:alnum:]]+)_([[:alnum:]]+)/?$");

const HOST: &'static str = "atcoder.jp";

impl AtCoderClient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Client for AtCoderClient {
    type Credential = Cred;

    fn is_contest_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(HOST)
            && RE_CONTEST_URL_PATH.is_match(url.path())
    }

    fn is_problem_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(HOST)
            && RE_PROBLEM_URL_PATH.is_match(url.path())
    }

    fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo> {
        todo!()
    }

    fn fetch_problem_info(&self, problem_url: &Url) -> Result<ProblemInfo> {
        todo!()
    }

    fn fetch_testcases(&self, problem_url: &Url) -> Result<Vec<Testcase>> {
        todo!()
    }

    fn login(&mut self, cred: &Self::Credential) -> Result<()> {
        todo!()
    }

    fn ask_credential(&self) -> Result<&Self::Credential> {
        todo!()
    }

    fn logout(&mut self) -> Result<()> {
        todo!()
    }

    fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<SubmissionID> {
        todo!()
    }
}
