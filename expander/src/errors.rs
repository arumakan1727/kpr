use std::path::PathBuf;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ExpanderError {
    #[error("expander: Unsupported language (given: {0})")]
    UnsupportedLang(PathBuf),

    #[error("expander: Filepath must be absolute (given: {0})")]
    NotAbsolutePath(PathBuf),

    #[error("{0}")]
    FileNotFound(String),
}

pub type Result<T> = ::std::result::Result<T, ExpanderError>;
