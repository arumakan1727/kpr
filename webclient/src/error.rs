use reqwest::StatusCode;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Wrong {fields}")]
    WrongCredential { fields: &'static str },

    #[error("Need login while accessing to {requested_url}")]
    NeedLogin { requested_url: String },

    #[error("Unexpected response code '{got}' (expected '{expected}') while requesting to {requested_url}")]
    UnexpectedResponseCode {
        got: StatusCode,
        expected: StatusCode,
        requested_url: String,
    },

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
