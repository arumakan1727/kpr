use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};

use crate::{config::RepoConfig, fsutil};

use super::{Vault, Workspace};

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
        assert!(repo_root.is_absolute());

        let to_abspath = |path: PathBuf| {
            if path.is_absolute() {
                path
            } else {
                repo_root.join(strip_prefix_dot(&path))
            }
        };
        cfg.vault_home = to_abspath(cfg.vault_home);
        cfg.workspace_home = to_abspath(cfg.workspace_home);
        cfg.workspace_template = to_abspath(cfg.workspace_template);

        Self {
            repo_root: repo_root.to_owned(),
            inner: cfg,
        }
    }

    pub fn from_config_file_finding_in_ancestors(
        current_dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let Some(config_filepath) = RepoConfig::find_file_in_ancestors(current_dir) else {
            bail!("Not in a kpr-repository dir: Cannot find '{}'", RepoConfig::FILENAME);
        };
        let cfg = {
            let toml = fsutil::read_to_string(&config_filepath).context("Cannot read a file")?;
            RepoConfig::from_toml(&toml).with_context(|| {
                format!("Invalid config TOML: {}", config_filepath.to_string_lossy())
            })?
        };
        let config_dir = config_filepath.parent().unwrap();
        Ok(Self::new(config_dir, cfg))
    }

    #[inline]
    pub fn vault(&self) -> Vault {
        Vault::new(&self.vault_home)
    }

    #[inline]
    pub fn workspace(&self) -> Workspace {
        Workspace::new(&self.workspace_home)
    }

    pub fn init_with_example_config(dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        if let Some(config_filepath) = RepoConfig::find_file_in_ancestors(dir) {
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

        let config_filepath = dir.join(RepoConfig::FILENAME);
        let toml = RepoConfig::example_toml();
        fsutil::write_with_mkdir(config_filepath, &toml)?;
        Ok(())
    }
}
