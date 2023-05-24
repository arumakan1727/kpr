use std::{fmt, io, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub action: ActionKind,
    pub path: PathBuf,
    pub source: io::Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    CreateDir,
    ReadFile,
    WriteFile,
    RemoveFile,
}

impl fmt::Display for ActionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ActionKind::*;
        let a = match self {
            CreateDir => "create dir",
            ReadFile => "read file",
            WriteFile => "write file",
            RemoveFile => "remove file",
        };
        write!(f, "{}", a)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Cannot {} '{}': {}",
            self.action,
            self.path.to_string_lossy(),
            self.source
        )
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source as &dyn std::error::Error)
    }
}
