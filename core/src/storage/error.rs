use std::{io, path::PathBuf};

pub type Result<T> = std::result::Result<T, self::Error>;

type Msg = &'static str;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} ({1}): {2}")]
    SingleIO(Msg, PathBuf, #[source] io::Error),

    #[error("Cannot create symlink (orig='{0}', link={1}): {2}")]
    Symlink(PathBuf, PathBuf, #[source] io::Error),

    #[error("Cannot serialize to JSON (dest='{0}'): {1}")]
    SerializeToJson(PathBuf, #[source] serde_json::Error),

    #[error("Cannot deserialize from JSON (src='{0}'): {1}")]
    DeserializeFromJson(PathBuf, #[source] serde_json::Error),
}
