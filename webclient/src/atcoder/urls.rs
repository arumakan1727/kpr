use crate::{problem_id, Platform, ProblemId, Url, UrlAnalyzer};
use ::lazy_regex::{lazy_regex, Lazy, Regex};

pub(super) static RE_CONTEST_URL_PATH: Lazy<Regex> = lazy_regex!(r"^/contests/([[:alnum:]]+)/?$");
pub(super) static RE_PROBLEM_URL_PATH: Lazy<Regex> =
    lazy_regex!(r"^/contests/([[:alnum:]]+)/tasks/(([[:alnum:]]+)_([[:alnum:]]+))/?$");

pub const DOMAIN: &str = "atcoder.jp";
pub const HOME_URL: &str = "https://atcoder.jp/home";
pub const LOGIN_URL: &str = "https://atcoder.jp/login";
pub const LOGOUT_URL: &str = "https://atcoder.jp/logout";
pub static TOP_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://atcoder.jp").unwrap());

pub struct AtCoderUrlAnalyzer;

impl UrlAnalyzer for AtCoderUrlAnalyzer {
    fn is_supported_origin(url: &Url) -> bool {
        match (url.scheme(), url.host_str(), url.port_or_known_default()) {
            ("https", Some(DOMAIN), Some(443)) => true,
            _ => false,
        }
    }

    fn is_contest_home_url(url: &Url) -> bool {
        Self::is_supported_origin(url) && RE_CONTEST_URL_PATH.is_match(url.path())
    }

    fn is_problem_url(url: &Url) -> bool {
        Self::is_supported_origin(url) && RE_PROBLEM_URL_PATH.is_match(url.path())
    }

    fn extract_problem_id(url: &Url) -> problem_id::Result<ProblemId> {
        use problem_id::Error;
        if !Self::is_supported_origin(url) {
            return Err(Error::UnknownOrigin(url.to_owned()));
        }
        let Some(caps) = RE_PROBLEM_URL_PATH.captures(url.path()) else {
            return Err(Error::NotProblemUrl(url.to_owned(), Platform::AtCoder))
        };
        Ok(ProblemId(caps[2].to_owned()))
    }
}
