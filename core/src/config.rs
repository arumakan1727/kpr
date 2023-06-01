use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use kpr_webclient::Platform;
use rust_embed::RustEmbed;
use serde::Deserialize;

use crate::serdable::GlobPattern;

pub fn authtoken_filename(platform: Platform) -> String {
    format!("{}-auth.json", platform.lowercase())
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub repository: RepoConfig,
    pub test: TestConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub vault_home: PathBuf,
    pub workspace_home: PathBuf,
    pub workspace_template: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestConfig {
    pub shell: PathBuf,
    pub include: GlobPattern,
    pub compile_before_run: bool,
    pub command: Vec<TestCommandConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TestCommandConfig {
    pub pattern: GlobPattern,

    #[serde(default = "Option::default")]
    pub compile: Option<String>,

    pub run: String,
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

    /// Find config file ancestor dirs, including current dir.
    pub fn find_file_in_ancestors(cur_dir: impl AsRef<Path>) -> Option<PathBuf> {
        let cur_dir = cur_dir.as_ref();
        cur_dir
            .ancestors()
            .map(|dir| dir.join(Self::FILENAME))
            .find(|path| path.is_file())
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
            repository: repo,
            test,
        } = cfg;

        assert_eq!(repo.vault_home, Path::new("./vault"));
        assert_eq!(repo.workspace_home, Path::new("./workspace"));
        assert_eq!(repo.workspace_template, Path::new("./template"));

        assert_eq!(test.shell, Path::new("/bin/sh"));
        assert_eq!(test.include, GlobPattern::parse("[mM]ain.*").unwrap());
        assert_eq!(test.compile_before_run, true);
        assert_eq!(test.command.len(), 2);
    }
}
