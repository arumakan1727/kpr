pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Local};
use error::*;
use kpr_webclient::problem_id::ProblemGlobalId;
use kpr_webclient::{ProblemMeta, Testcase, Url};

use crate::client::SessionPersistentClient;
use crate::config::TestConfig;
use crate::interactive::ask_credential;
use crate::storage::{
    workspace, ProblemVault, ProblemWorkspace, Repository, WorkspaceNameModifier,
};
use crate::testing::{AsyncTestcase, FsTestcase, JudgeCode, TestOutcome, TestRunner};

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
) -> Result<(ProblemVault, ProblemMeta, Vec<Testcase>)> {
    ensure!(cli.is_problem_url(url), "Not a problem url: {}", url);

    let (problem_meta, testcases) = cli
        .fetch_problem_detail(url)
        .await
        .context("Failed to fetch testcase")?;

    let vault = repo.vault_home();

    let saved_location = vault
        .save_problem_data(&problem_meta, &testcases)
        .context("Failed to save problem data")?;

    Ok((saved_location, problem_meta, testcases))
}

pub async fn ensure_problem_data_saved(
    cli: &SessionPersistentClient,
    url: &Url,
    repo: &Repository,
) -> Result<(ProblemVault, ProblemMeta)> {
    ensure!(cli.is_problem_url(url), "Not a problem url: {}", url);

    let platform = cli.platform();
    let problem_id = cli.extract_problem_id(url).unwrap();
    let vault = repo.vault_home();

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
) -> Result<ProblemWorkspace> {
    ensure!(
        cli.is_problem_url(problem_url),
        "Not a problem url: {}",
        problem_url
    );

    let (saved_location, meta) = self::ensure_problem_data_saved(cli, &problem_url, repo).await?;

    let problem_id = ProblemGlobalId::new(meta.platform, meta.problem_id);
    let loc = repo
        .workspace_home()
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

pub async fn create_contest_workspace(
    cli: &SessionPersistentClient,
    contest_url: &Url,
    repo: &Repository,
    today: DateTime<Local>,
) -> Result<Vec<ProblemWorkspace>> {
    ensure!(
        cli.is_contest_home_url(contest_url),
        "Not a contest url: {}",
        contest_url,
    );
    let contest = cli
        .fetch_contest_info(contest_url)
        .await
        .with_context(|| format!("Failed to fetch contest info (url={})", contest_url))?;

    let serial_code = if contest.problems.len() <= 26 {
        // 1 => "a",  2 => "b",  3 => "c", ...
        |ord: u32| ((b'a' + (ord - 1) as u8) as char).to_string()
    } else {
        |ord: u32| format!("{:02}", ord)
    };

    let w = repo.workspace_home();
    let mut workspace_locations = Vec::new();

    for problem in &contest.problems {
        // Avoid Dos attack
        std::thread::sleep(Duration::from_millis(200));

        let url = Url::parse(&problem.url).with_context(|| {
            format!(
                "Failed to get correct problem url (contest_url={})",
                contest_url
            )
        })?;

        let (vault_loc, _meta) = self::ensure_problem_data_saved(cli, &url, repo).await?;
        let loc = w
            .create_workspace(
                &vault_loc,
                &repo.workspace_template,
                WorkspaceNameModifier {
                    today,
                    category: &contest.short_title,
                    name: &serial_code(problem.ord),
                },
            )
            .context("Failed to create contest workspace")?;
        println!(
            "Successfully created workspace {}",
            loc.dir().to_string_lossy()
        );
        workspace_locations.push(loc);
    }
    Ok(workspace_locations)
}
