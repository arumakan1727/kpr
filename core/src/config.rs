use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use anyhow::{bail, Context};
use kpr_webclient::Platform;
use rust_embed::RustEmbed;
use serde::Deserialize;

use crate::fsutil;

pub const PROBLEM_METADATA_FILENAME: &str = ".problem.json";
pub const REPOSITORY_CONFIG_FILENAME: &str = "kpr-repository.toml";
pub const VAULT_TESTCASE_DIR_NAME: &str = "testcase";

pub fn authtoken_filename(platform: Platform) -> String {
    format!("{}-auth.json", platform.lowercase())
}

pub fn problem_dir(
    dir: impl AsRef<Path>,
    p: Platform,
    problem_unique_name: impl AsRef<str>,
) -> PathBuf {
    dir.as_ref()
        .join(p.lowercase())
        .join(problem_unique_name.as_ref())
}

/// Returns tuple (input_filename, output_filename).
///
/// ```
/// use kpr_core::config::testcase_filename;
///
/// let (infile, outfile) = testcase_filename(1);
/// assert_eq!(infile, "in1.txt");
/// assert_eq!(outfile, "out1.txt");
/// ```
pub fn testcase_filename(ord: u32) -> (String, String) {
    (format!("in{}.txt", ord), format!("out{}.txt", ord))
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub vault_home: PathBuf,
    pub daily_home: PathBuf,
    pub solvespace_template: PathBuf,
}

#[derive(Debug, Clone)]
pub struct QualifiedRepoConfig {
    inner: RepoConfig,
    pub repo_root: PathBuf,
}

impl Deref for QualifiedRepoConfig {
    type Target = RepoConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Asset;

impl RepoConfig {
    pub fn example_toml() -> String {
        let file = Asset::get(REPOSITORY_CONFIG_FILENAME).unwrap();
        std::str::from_utf8(file.data.as_ref()).unwrap().to_owned()
    }

    pub fn from_toml(s: &str) -> StdResult<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Find REPOSITORY_CONFIG_FILENAME from current_dir to ancestor dirs.
    pub fn find_filepath(current_dir: impl AsRef<Path>) -> Option<PathBuf> {
        assert!(current_dir.as_ref().is_absolute());

        let mut current_dir = Some(current_dir.as_ref());
        while let Some(dir) = current_dir {
            let filepath = dir.join(REPOSITORY_CONFIG_FILENAME);
            if filepath.is_file() {
                return Some(filepath.to_owned());
            }
            current_dir = dir.parent();
        }
        None
    }

    pub fn with_root_dir(mut self, root_dir: impl AsRef<Path>) -> QualifiedRepoConfig {
        let root_dir = root_dir.as_ref();
        self.vault_home = root_dir.join(self.vault_home);
        self.daily_home = root_dir.join(self.daily_home);
        self.solvespace_template = root_dir.join(self.solvespace_template);
        QualifiedRepoConfig {
            repo_root: root_dir.to_owned(),
            inner: self,
        }
    }
}

impl QualifiedRepoConfig {
    pub fn from_fs(current_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let Some(config_path) = RepoConfig::find_filepath(current_dir) else {
            bail!("Not in a kpr-repository dir: Cannot find '{}'", REPOSITORY_CONFIG_FILENAME);
        };
        let contents = fsutil::read_to_string(&config_path).context("Failed to read")?;
        let cfg = RepoConfig::from_toml(&contents)
            .with_context(|| format!("Invalid TOML: {}", config_path.to_string_lossy()))?;
        Ok(cfg.with_root_dir(config_path.parent().unwrap()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_toml_should_be_parsable() {
        let toml = RepoConfig::example_toml();
        let r = RepoConfig::from_toml(&toml).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        assert_eq!(r.vault_home, Path::new("./vault"));
        assert_eq!(r.daily_home, Path::new("./daily"));
        assert_eq!(r.solvespace_template, Path::new("./template"));
    }
}
