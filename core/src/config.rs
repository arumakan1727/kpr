use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use anyhow::Context as _;
use kpr_webclient::Platform;
use rust_embed::RustEmbed;
use serde::Deserialize;

use crate::serdable::GlobPattern;
use crate::testing::runner::TestCommand;

pub fn authtoken_filename(platform: Platform) -> String {
    format!("{}-auth.json", platform.lowercase())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub source_config_file: Option<PathBuf>,
    pub repository: RepoConfig,
    pub test: TestConfig,
    pub submit: SubmissionConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RepoConfig {
    pub vault_home: PathBuf,
    pub workspace_home: PathBuf,
    pub workspace_template: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TestConfig {
    pub shell: PathBuf,
    pub include: GlobPattern,
    pub compile_before_run: bool,
    pub stdout_capture_max_bytes: usize,
    pub stderr_capture_max_bytes: usize,
    pub command: Vec<TestCommandConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TestCommandConfig {
    pub pattern: GlobPattern,
    pub compile: Option<String>,
    pub run: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SubmissionConfig {
    pub run_test: bool,
    pub lang: SubmissionLangConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SubmissionLangConfig {
    pub atcoder: Vec<SubmissionLangConfigEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SubmissionLangConfigEntry {
    pub pattern: GlobPattern,
    pub lang: String,
}

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Asset;

impl Config {
    pub const FILENAME: &str = "kpr.toml";

    pub fn example_toml() -> String {
        let file = Asset::get(Self::FILENAME).unwrap();
        std::str::from_utf8(file.data.as_ref()).unwrap().to_owned()
    }

    pub fn from_toml(s: &str) -> StdResult<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn from_toml_file(filepath: PathBuf) -> anyhow::Result<Self> {
        let toml = fsutil::read_to_string(&filepath).context("Cannot read a file")?;
        let mut cfg = Self::from_toml(&toml)
            .with_context(|| format!("Invalid config TOML: {:?}", filepath))?;
        cfg.source_config_file = Some(filepath);
        Ok(cfg)
    }

    /// Find config file ancestor dirs, including current dir.
    pub fn find_file_in_ancestors(cur_dir: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let cur_dir = cur_dir.as_ref();
        cur_dir
            .ancestors()
            .map(|dir| dir.join(Self::FILENAME))
            .find(|path| path.is_file())
            .with_context(|| {
                format!(
                    "Not in a kpr-repository dir: Cannot find '{}'",
                    Self::FILENAME
                )
            })
    }

    pub fn from_file_finding_in_ancestors(cur_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_filepath = Config::find_file_in_ancestors(cur_dir)?;
        Self::from_toml_file(config_filepath)
    }
}

impl TestConfig {
    pub fn find_test_cmd_for_filename(&self, filename: impl AsRef<str>) -> Option<TestCommand> {
        self.command
            .iter()
            .find(|entry| entry.pattern.matches(filename.as_ref()))
            .map(|entry| TestCommand {
                compile: entry.compile.to_owned(),
                run: entry.run.to_owned(),
            })
    }
}

impl SubmissionLangConfig {
    pub fn get(&self, platform: Platform) -> &[SubmissionLangConfigEntry] {
        use Platform::*;
        match platform {
            AtCoder => &self.atcoder,
        }
    }

    pub fn find_submission_lang_for_filename(
        &self,
        filename: impl AsRef<str>,
        platform: Platform,
    ) -> Option<&str> {
        let filename = filename.as_ref();
        self.get(platform)
            .iter()
            .find(|entry| entry.pattern.matches(filename))
            .map(|entry| entry.lang.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn example_toml_should_be_parsable() {
        let toml = Config::example_toml();
        let cfg = dbg!(Config::from_toml(&toml)).unwrap();

        let Config {
            source_config_file,
            repository: repo,
            test,
            submit,
        } = cfg;

        assert_eq!(source_config_file, None);
        assert_eq!(repo.vault_home, Path::new("./vault"));
        assert_eq!(repo.workspace_home, Path::new("./workspace"));
        assert_eq!(repo.workspace_template, Path::new("./template"));

        assert_eq!(test.shell, Path::new("/bin/sh"));
        assert_eq!(test.include, GlobPattern::parse("[mM]ain.*").unwrap());
        assert_eq!(test.compile_before_run, true);
        assert_eq!(test.command.len(), 3);

        assert_eq!(submit.run_test, true);
        assert_eq!(submit.lang.atcoder.len(), 3);
        assert_eq!(
            submit.lang.atcoder[0],
            SubmissionLangConfigEntry {
                pattern: GlobPattern::parse("*.cpp").unwrap(),
                lang: "C++ (GCC 9.2.1)".to_owned(),
            }
        );
    }
}
