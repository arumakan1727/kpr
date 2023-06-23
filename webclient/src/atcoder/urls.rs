use crate::{problem_id, Platform, ProblemId, Url, UrlAnalyzer};
use ::lazy_regex::{lazy_regex, Lazy, Regex};

pub(super) static RE_CONTEST_URL_PATH: Lazy<Regex> = lazy_regex!(r"^/contests/([0-9A-Za-z_-]+)/?$");
pub(super) static RE_PROBLEM_URL_PATH: Lazy<Regex> =
    lazy_regex!(r"^/contests/([0-9A-Za-z_-]+)/tasks/(([0-9A-Za-z_-]+)_([[:alnum:]]+))/?$");

pub(super) static RE_PROBLEMS_VIRTUAL_CONTEST_URL_FRAGMENT: Lazy<Regex> =
    lazy_regex!(r"^/contest/show/([a-zA-Z0-9-]+)");

pub const DOMAIN: &str = "atcoder.jp";
pub const DOMAIN_KENKOOOO: &str = "kenkoooo.com";
pub const HOME_URL: &str = "https://atcoder.jp/home";
pub const LOGIN_URL: &str = "https://atcoder.jp/login";
pub const LOGOUT_URL: &str = "https://atcoder.jp/logout";
pub static TOP_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://atcoder.jp").unwrap());

pub struct AtCoderUrlAnalyzer;

impl AtCoderUrlAnalyzer {
    pub fn is_atcoder(url: &Url) -> bool {
        Self::is_https(url) && url.domain() == Some(DOMAIN)
    }

    pub fn is_atcoder_problems(url: &Url) -> bool {
        Self::is_https(url)
            && url.domain() == Some(DOMAIN_KENKOOOO)
            && url.path().starts_with("/atcoder/")
    }

    pub fn is_atcoder_contest_home_url(url: &Url) -> bool {
        Self::is_atcoder(url) && RE_CONTEST_URL_PATH.is_match(url.path())
    }

    pub fn is_problems_virtual_contest_url(url: &Url) -> bool {
        Self::is_atcoder_problems(url)
            && RE_PROBLEMS_VIRTUAL_CONTEST_URL_FRAGMENT.is_match(url.fragment().unwrap_or(""))
    }
}

impl UrlAnalyzer for AtCoderUrlAnalyzer {
    fn is_supported_url(url: &Url) -> bool {
        Self::is_atcoder(url) || Self::is_atcoder_problems(url)
    }

    fn is_contest_home_url(url: &Url) -> bool {
        Self::is_atcoder_contest_home_url(url) || Self::is_problems_virtual_contest_url(url)
    }

    fn is_problem_url(url: &Url) -> bool {
        Self::is_atcoder(url) && RE_PROBLEM_URL_PATH.is_match(url.path())
    }

    fn extract_problem_id(url: &Url) -> problem_id::Result<ProblemId> {
        use problem_id::Error;
        if !Self::is_atcoder(url) {
            return Err(Error::UnknownOrigin(url.to_owned()));
        }
        let Some(caps) = RE_PROBLEM_URL_PATH.captures(url.path()) else {
            return Err(Error::NotProblemUrl(url.to_owned(), Platform::AtCoder))
        };
        Ok(ProblemId(caps[2].to_owned()))
    }
}
