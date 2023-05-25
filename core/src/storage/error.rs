use std::{fmt, path::PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub action: ActionKind,
    pub path: PathBuf,
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    CreateDir,
    ReadFile,
    WriteFile,
    RemoveFile,
    SerializeToJson,
    DeserializeFromJson,
}

impl fmt::Display for ActionKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ActionKind::*;
        let a = match self {
            CreateDir => "create dir",
            ReadFile => "read file",
            WriteFile => "write file",
            RemoveFile => "remove file",
            SerializeToJson => "serialize to json",
            DeserializeFromJson => "deserialize from json",
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
        Some(&*self.source)
    }
}
