pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
use std::path::{Path, PathBuf};

use error::*;
use kpr_webclient::{ProblemMeta, Testcase, Url};

use crate::client::SessionPersistentClient;
use crate::config::{QualifiedRepoConfig, RepoConfig};
use crate::interactive::ask_credential;
use crate::repository::Vault;
use crate::{config, fsutil};

pub async fn login(cli: &mut SessionPersistentClient) -> Result<()> {
    ensure!(
        !cli.is_logged_in(),
        "Already logged in to {}",
        cli.platform()
    );

    let cred = ask_credential(cli.credential_fields());

    cli.login(cred)
        .await
        .with_context(|| format!("Failed to login to {}", cli.platform()))?;

    cli.save_authtoken_to_storage()
}

pub async fn logout(cli: &mut SessionPersistentClient) -> Result<()> {
    ensure!(
        cli.is_logged_in(),
        "Already logged out from {}",
        cli.platform()
    );

    let _ = cli.remove_authtoken_from_storagr();

    cli.logout()
        .await
        .with_context(|| format!("Failed to logout from {}", cli.platform()))
}

pub fn init_kpr_repository(dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();

    let config_path = dir.join(config::REPOSITORY_CONFIG_FILENAME);
    let toml = RepoConfig::example_toml();
    fsutil::write_with_mkdir(config_path, &toml)?;

    let r = RepoConfig::from_toml(&toml).unwrap();

    let example_template_filepath = dir.join(&r.workspace_template).join("main.cpp");
    let template_code = r#"#include <bits/stdc++.h>
using namespace std;

int main() {
    cout << "Hello world" << endl;
}
"#;
    fsutil::write_with_mkdir(example_template_filepath, template_code)?;
    Ok(())
}

/// Returns (saved_problem_dir_path, metadata, testcases)
pub async fn save_problem_data(
    cli: &SessionPersistentClient,
    url: &Url,
    repo: &QualifiedRepoConfig,
) -> Result<(PathBuf, ProblemMeta, Vec<Testcase>)> {
    ensure!(cli.is_problem_url(url), "{} is not a problem url", url);

    let (problem_meta, testcases) = cli
        .fetch_problem_detail(url)
        .await
        .context("Failed to fetch testcase")?;
    let vault = Vault::new(&repo.vault_home);
    let platform = cli.platform();
    let problem_id = &problem_meta.global_id;
    vault
        .save_problem_metadata(&problem_meta)
        .context("Failed to save problem metadata")?;
    vault
        .save_testcases(&testcases, platform, problem_id)
        .context("Failed to save testcase")?;
    Ok((
        vault.resolve_problem_dir(platform, problem_id),
        problem_meta,
        testcases,
    ))
}

pub async fn ensure_problem_data_saved(
    cli: &SessionPersistentClient,
    url: &Url,
    repo: &QualifiedRepoConfig,
) -> Result<(PathBuf, ProblemMeta)> {
    ensure!(cli.is_problem_url(url), "{} is not a problem url", url);

    let platform = cli.platform();
    let problem_id = cli.problem_global_id(url).unwrap();
    let vault = Vault::new(&repo.vault_home);
    if let Ok(problem_meta) = vault.load_problem_metadata(platform, &problem_id) {
        return Ok((
            vault.resolve_problem_dir(platform, &problem_id),
            problem_meta,
        ));
    }
    self::save_problem_data(cli, url, repo)
        .await
        .map(|(dir, problem_meta, _testcases)| (dir, problem_meta))
}
