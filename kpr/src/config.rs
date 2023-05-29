use serde::{Deserialize, Serialize};
use std::{fs::File, io, path::PathBuf};

use crate::{cmd::GlobalArgs, util};

pub const APP_NAME: &str = "kpr-cli";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_cache_dir")]
    pub cache_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cache_dir: Self::default_cache_dir(),
        }
    }
}

impl Config {
    pub const FILENAME: &str = "kpr-cli.toml";

    pub fn filepath() -> PathBuf {
        let dir = dirs::config_dir().expect("Failed to get user's config dir path");
        dir.join(APP_NAME).join(Self::FILENAME)
    }

    fn default_cache_dir() -> PathBuf {
        let dir = dirs::cache_dir().expect("Failed to get user's cache dir path");
        dir.join(APP_NAME)
    }

    pub fn from_file_or_default() -> Self {
        let path = Self::filepath();
        let toml_str = match File::open(&path).and_then(io::read_to_string) {
            Ok(toml) => toml,
            _ => return Config::default(),
        };
        toml::from_str(&toml_str).unwrap_or_else(|e| {
            eprintln!(
                "[Warn] Invalid config '{}': {:#}",
                util::replace_homedir_to_tilde(path).to_string_lossy(),
                e
            );
            Self::default()
        })
    }

    pub fn with_args(mut self, args: &GlobalArgs) -> Self {
        let GlobalArgs {
            subcmd: _,
            cache_dir,
        } = args;

        cache_dir.as_ref().map(|d| self.cache_dir = d.clone());
        self
    }

    pub fn from_file_and_args(args: &GlobalArgs) -> Self {
        Self::from_file_or_default().with_args(args)
    }
}
