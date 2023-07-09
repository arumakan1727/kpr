use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::bail;

use crate::config::{Config, RepoConfig};

use super::{VaultHome, WorkspaceHome};

#[derive(Debug, Clone)]
pub struct Repository {
    inner: RepoConfig,
}

impl Deref for Repository {
    type Target = RepoConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<Config> for Repository {
    fn from(c: Config) -> Self {
        Self::new(c.repository)
    }
}

fn strip_prefix_dot(path: &Path) -> &Path {
    path.strip_prefix(".").unwrap_or(path)
}

impl Repository {
    pub fn new(mut cfg: RepoConfig) -> Self {
        let repo_root = &cfg.source_config_dir;

        let with_repo_root = |path: PathBuf| {
            if path.is_absolute() {
                path
            } else {
                repo_root.join(strip_prefix_dot(&path))
            }
        };
        cfg.vault_home = with_repo_root(cfg.vault_home);
        cfg.workspace_home = with_repo_root(cfg.workspace_home);
        cfg.workspace_template = with_repo_root(cfg.workspace_template);

        Self { inner: cfg }
    }

    #[inline]
    pub fn vault_home(&self) -> VaultHome {
        VaultHome::new(&self.vault_home)
    }

    #[inline]
    pub fn workspace_home(&self) -> WorkspaceHome {
        WorkspaceHome::new(&self.workspace_home)
    }

    pub fn init_with_example_config(dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        if let Ok(config_filepath) = Config::find_file_in_ancestors(dir) {
            let path = if config_filepath.is_relative() && !config_filepath.starts_with("./") {
                Path::new("./").join(config_filepath)
            } else {
                config_filepath
            };
            bail!(
                "Already being kpr-repository.\nIf it's intentional, remove {:?} and then try again.",
                path
            );
        }

        let config_filepath = dir.join(Config::FILENAME);
        let toml = Config::example_toml();
        fsutil::write_with_mkdir(config_filepath, &toml)?;
        Ok(())
    }
}
