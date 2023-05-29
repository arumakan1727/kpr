use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use kpr_webclient::Platform;
use rust_embed::RustEmbed;
use serde::Deserialize;

pub fn authtoken_filename(platform: Platform) -> String {
    format!("{}-auth.json", platform.lowercase())
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub vault_home: PathBuf,
    pub workspace_home: PathBuf,
    pub workspace_template: PathBuf,
}

#[derive(RustEmbed)]
#[folder = "examples/assets/"]
struct Asset;

impl RepoConfig {
    pub const FILENAME: &str = "kpr-repository.toml";

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
        let toml = RepoConfig::example_toml();
        let cfg = RepoConfig::from_toml(&toml).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        assert_eq!(cfg.vault_home, Path::new("./vault"));
        assert_eq!(cfg.workspace_home, Path::new("./workspace"));
        assert_eq!(cfg.workspace_template, Path::new("./template"));
    }
}
