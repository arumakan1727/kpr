use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use std::{fs::File, io, path::PathBuf, process};

use crate::cmd::GlobalArgs;

pub const APP_NAME: &'static str = "kpr-cli";
pub const CONFIG_FILE_NAME: &'static str = "kpr-cli.toml";

pub fn config_file_path() -> PathBuf {
    let dir = dirs::config_dir().expect("Failed to get user's config dir path");
    dir.join(APP_NAME).join(CONFIG_FILE_NAME)
}

fn default_cache_dir() -> PathBuf {
    let dir = dirs::cache_dir().expect("Failed to get user's cache dir path");
    dir.join(APP_NAME)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            cache_dir: default_cache_dir(),
        }
    }
}

impl Config {
    pub fn from_file() -> anyhow::Result<Self> {
        let path = config_file_path();
        let toml_str = match File::open(&path).map(io::read_to_string) {
            Ok(Ok(toml)) => toml,
            _ => return Ok(Config::default()),
        };
        toml::from_str(&toml_str)
            .with_context(|| format!("Invalid toml content: {}", path.to_string_lossy()))
    }

    pub fn with_args(mut self, args: &GlobalArgs) -> Self {
        let GlobalArgs {
            subcmd: _,
            cache_dir,
        } = args;

        cache_dir.as_ref().map(|d| self.cache_dir = d.clone());
        self
    }

    pub fn from_file_and_args_or_die(args: &GlobalArgs) -> Self {
        match Self::from_file() {
            Ok(cfg) => cfg.with_args(args),
            Err(e) => {
                eprintln!("Config error: {}", e);
                process::exit(1);
            }
        }
    }
}
