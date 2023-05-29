pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
use std::path::Path;

use chrono::{DateTime, Local};
use error::*;
use kpr_webclient::problem_id::ProblemGlobalId;
use kpr_webclient::{ProblemMeta, Testcase, Url};

use crate::client::SessionPersistentClient;
use crate::interactive::ask_credential;
use crate::storage::{
    ProblemVaultLocation, ProblemWorkspaceLocation, Repository, WorkspaceNameModifier,
};

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

    let _ = cli.remove_authtoken_from_storage();

    cli.logout()
        .await
        .with_context(|| format!("Failed to logout from {}", cli.platform()))
}

pub fn init_kpr_repository(dir: impl AsRef<Path>) -> Result<()> {
    Repository::init_with_example_config(dir).context("Failed to init kpr repository")
}

/// Returns (saved_problem_dir_path, metadata, testcases)
pub async fn fetch_and_save_problem_data(
    cli: &SessionPersistentClient,
    url: &Url,
    repo: &Repository,
) -> Result<(ProblemVaultLocation, ProblemMeta, Vec<Testcase>)> {
    ensure!(cli.is_problem_url(url), "{} is not a problem url", url);

    let (problem_meta, testcases) = cli
        .fetch_problem_detail(url)
        .await
        .context("Failed to fetch testcase")?;

    let vault = repo.vault();

    let saved_location = vault
        .save_problem_data(&problem_meta, &testcases)
        .context("Failed to save problem data")?;

    Ok((saved_location, problem_meta, testcases))
}

pub async fn ensure_problem_data_saved(
    cli: &SessionPersistentClient,
    url: &Url,
    repo: &Repository,
) -> Result<(ProblemVaultLocation, ProblemMeta)> {
    ensure!(cli.is_problem_url(url), "Not a problem url: {}", url);

    let platform = cli.platform();
    let problem_id = cli.extract_problem_id(url).unwrap();
    let vault = repo.vault();

    if let Ok((loc, problem_meta)) = vault.load_problem_metadata(platform, &problem_id) {
        return Ok((loc, problem_meta));
    }
    self::fetch_and_save_problem_data(cli, url, repo)
        .await
        .map(|(dir, problem_meta, _testcases)| (dir, problem_meta))
}

pub async fn create_shojin_workspace(
    cli: &SessionPersistentClient,
    problem_url: &Url,
    repo: &Repository,
    today: DateTime<Local>,
) -> Result<ProblemWorkspaceLocation> {
    ensure!(
        cli.is_problem_url(problem_url),
        "Not a problem url: {}",
        problem_url
    );

    let (saved_location, meta) = self::ensure_problem_data_saved(cli, &problem_url, repo).await?;

    let problem_id = ProblemGlobalId::new(meta.platform, meta.problem_id);
    let loc = repo
        .workspace()
        .create_workspace(
            &saved_location,
            &repo.workspace_template,
            WorkspaceNameModifier {
                today,
                category: "shojin",
                name: &problem_id.to_string(),
            },
        )
        .context("Failed to create shojin workspace")?;
    Ok(loc)
}


    };
}
