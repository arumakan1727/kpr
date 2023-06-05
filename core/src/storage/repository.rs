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
    pub repo_root: PathBuf,
}

impl Deref for Repository {
    type Target = RepoConfig;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn strip_prefix_dot(path: &Path) -> &Path {
    path.strip_prefix(".").unwrap_or(path)
}

impl Repository {
    pub fn new(repo_root: impl AsRef<Path>, mut cfg: RepoConfig) -> Self {
        let repo_root = repo_root.as_ref();

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

        Self {
            repo_root: repo_root.to_owned(),
            inner: cfg,
        }
    }

    pub fn from_config_file_finding_in_ancestors(
        cur_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let cfg = Config::from_file_finding_in_ancestors(cur_dir)?;
        let config_filepath = cfg.source_config_file.unwrap();
        let config_dir = config_filepath.parent().unwrap_or(Path::new("."));
        Ok(Self::new(config_dir, cfg.repository))
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
                "Already being kpr-repository. (found config: '{}')",
                path.to_string_lossy()
            );
        }

        let config_filepath = dir.join(Config::FILENAME);
        let toml = Config::example_toml();
        fsutil::write_with_mkdir(config_filepath, &toml)?;
        Ok(())
    }
}

impl From<&Config> for Repository {
    fn from(c: &Config) -> Self {
        let config_dir = c
            .source_config_file
            .as_ref()
            .unwrap()
            .parent()
            .unwrap_or(Path::new("."));
        Self::new(&config_dir, c.repository.clone())
    }
}
