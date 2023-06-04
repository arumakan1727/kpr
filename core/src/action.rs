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
use kpr_webclient::{PgLang, ProblemInfo, SampleTestcase, Url};

use crate::client::SessionPersistentClient;
use crate::config::{SubmissionConfig, TestConfig};
use crate::fsutil;
use crate::interactive::ask_credential;
use crate::storage::{
    workspace, PlatformVault, ProblemVault, ProblemWorkspace, Repository, WorkspaceNameModifier,
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
) -> Result<(ProblemVault, ProblemInfo, Vec<SampleTestcase>)> {
    ensure!(cli.is_problem_url(url), "Not a problem url: {}", url);

    let (problem_info, testcases) = cli
        .fetch_problem_detail(url)
        .await
        .context("Failed to fetch testcase")?;

    let vault = repo.vault_home();

    let saved_location = vault
        .save_problem_data(&problem_info, &testcases)
        .context("Failed to save problem data")?;

    Ok((saved_location, problem_info, testcases))
}

pub async fn ensure_problem_data_saved(
    cli: &SessionPersistentClient,
    url: &Url,
    repo: &Repository,
) -> Result<(ProblemVault, ProblemInfo)> {
    ensure!(cli.is_problem_url(url), "Not a problem url: {}", url);

    let platform = cli.platform();
    let problem_id = cli.extract_problem_id(url).unwrap();
    let vault = repo.vault_home();

    if let Ok((loc, problem_info)) = vault.load_problem_info(platform, &problem_id) {
        return Ok((loc, problem_info));
    }
    self::fetch_and_save_problem_data(cli, url, repo)
        .await
        .map(|(dir, problem_info, _testcases)| (dir, problem_info))
}

pub async fn fetch_and_save_submittable_lang_list(
    cli: &SessionPersistentClient,
    repo: &Repository,
) -> Result<(PlatformVault, Vec<PgLang>)> {
    let langs = cli
        .fetch_submittable_language_list()
        .await
        .context("Failed to fetch submittable language list")?;

    let vault = repo.vault_home();
    let saved_location = vault
        .save_submittable_lang_list(cli.platform(), &langs)
        .context("Failed to save submittable language list")?;

    Ok((saved_location, langs))
}

pub async fn ensure_submittable_lang_list_saved(
    cli: &SessionPersistentClient,
    repo: &Repository,
) -> Result<(PlatformVault, Vec<PgLang>)> {
    let vault = repo.vault_home();
    if let Ok((loc, langs)) = vault.load_submittable_lang_list(cli.platform()) {
        return Ok((loc, langs));
    }
    self::fetch_and_save_submittable_lang_list(cli, repo).await
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

    let (saved_location, info) = self::ensure_problem_data_saved(cli, &problem_url, repo).await?;

    let problem_id = ProblemGlobalId::new(info.platform, info.problem_id);
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

        let (vault_loc, _info) = self::ensure_problem_data_saved(cli, &url, repo).await?;
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

pub async fn do_test(
    program_file: impl AsRef<Path>,
    testcase_dir: impl AsRef<Path>,
    cfg: &TestConfig,
) -> Result<Vec<TestOutcome>> {
    let testcases = FsTestcase::enumerate(&testcase_dir, &workspace::TestcaseFinder)
        .context("Failed to find testcase")?;
    if testcases.is_empty() {
        bail!(
            "No testcases is saved in {}",
            testcase_dir.as_ref().to_string_lossy()
        );
    }

    let filename = program_file.as_ref().file_name().unwrap().to_string_lossy();
    let cmd = cfg.find_test_cmd_for_filename(&filename).with_context(|| {
        format!(
            "Unconfigured test command for filename '{}' (No entry matched glob in `test.command[]`)",
            filename
        )
    })?;

    let runner = TestRunner::new(cmd)
        .shell(cfg.shell.to_owned())
        .program_file(&program_file)?;

    if cfg.compile_before_run && runner.is_compile_cmd_defined() {
        let cmd = runner.get_command().compile.as_ref().unwrap();
        println!("Compiling {}\n{}", filename, cmd);
        runner.compile().await?;
    }

    println!("Run command: {}", runner.get_command().run);

    let mut results = Vec::with_capacity(testcases.len());
    for t in &testcases {
        print!("Running testcase {} ... ", t.name());

        let res = runner.run(t).await?;
        println!("{} {:?}", res.judge, res.execution_time);

        if res.judge != JudgeCode::AC && res.output.is_some() {
            let o = res.output.as_ref().unwrap();
            let bold_line = "=".repeat(50);
            let dash_line = " -".repeat(10);
            println!("{}", bold_line);
            println!("{} stdout{}\n{}", dash_line, dash_line, o.stdout);
            println!("{} stderr{}\n{}", dash_line, dash_line, o.stderr);
            println!("{}", bold_line);
        }

        results.push(res);
    }
    Ok(results)
}

pub async fn submit(
    cli: &SessionPersistentClient,
    program_file: impl AsRef<Path>,
    problem_url: &Url,
    cfg: &SubmissionConfig,
    available_langs: &[PgLang],
) -> Result<()> {
    let platform = cli.platform();
    let filename = program_file.as_ref().file_name().unwrap().to_string_lossy();

    let lang = {
        let lang_name = cfg
            .lang
            .find_submission_lang_for_filename(&filename, platform)
            .with_context(|| format!("Unconfigured submission lang for filename '{}' (No entry mathed glob in `submit.lang.{}[]`)", filename, platform.lowercase()))?;

        available_langs
            .iter()
            .find(|x| x.name == lang_name)
            .with_context(|| format!("No such language named '{}'", lang_name))?
    };

    let source_code = fsutil::read_to_string(&program_file)?;

    cli.submit(problem_url, &lang, &source_code)
        .await
        .with_context(|| {
            format!(
                "Failed to submit {:?} with specifying lang='{}' (langID={})",
                program_file.as_ref(),
                lang.name,
                lang.id
            )
        })?;

    Ok(())
}
