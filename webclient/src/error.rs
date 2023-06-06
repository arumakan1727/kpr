use reqwest::StatusCode;
use scraper::Selector;
use url::Url;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Wrong {fields}")]
    WrongCredential { fields: &'static str },

    #[error("Need login while accessing to {requested_url}")]
    NeedLogin { requested_url: String },

    #[error("Failed to parse as URL '{url}'")]
    InvalidSyntaxUrl {
        url: String,

        #[source]
        source: url::ParseError,
    },

    #[error("Not a contest URL '{0}'")]
    NotContestUrl(Url),

    #[error("Unexpected response code '{got}' (expected '{expected}') while requesting to {requested_url}")]
    UnexpectedResponseCode {
        got: StatusCode,
        expected: StatusCode,
        requested_url: String,
    },

    #[error("No such html element (selector: {0:?})")]
    NoSuchElementMatchesToSelector(Selector),

    #[error("No such attr named '{0}' (selector: {1:?})")]
    NoSuchAttr(&'static str, Selector),

    #[error("Element has no inner text (selector: {0:?})")]
    NoInnerText(Selector),

    #[error("Unexpected redirect path '{got}' (expected '{expected}') while accessing to {requested_url}")]
    UnexpectedRedirectPath {
        got: String,
        expected: String,
        requested_url: String,
    },

    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
