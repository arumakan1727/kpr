pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}

use std::{path::Path, time::Duration};

use chrono::{DateTime, Local};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kpr_webclient::{problem_id::ProblemGlobalId, PgLang, ProblemInfo, SampleTestcase, Url};

use self::error::*;
use crate::{
    client::SessionPersistentClient,
    config::{SubmissionConfig, TestConfig},
    interactive::{ask_credential, SpinnerExt as _},
    storage::{
        workspace, PlatformVault, ProblemVault, ProblemWorkspace, Repository, WorkspaceNameModifier,
    },
    style,
    testing::{AsyncTestcase, FsTestcase, JudgeCode, TestCommand, TestOutcome, TestRunner},
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
) -> Result<Vec<(ProblemWorkspace, ProblemInfo)>> {
    ensure!(
        cli.is_contest_home_url(contest_url),
        "Not a contest url: {}",
        contest_url,
    );

    let progress_spinner = ProgressBar::new(1)
        .with_style(ProgressStyle::with_template(" {prefix} {spinner} {wide_msg}").unwrap())
        .with_prefix("🔍")
        .with_message("Fetching contest info ...")
        .with_ticking();

    let contest = cli
        .fetch_contest_info(contest_url)
        .await
        .with_context(|| format!("Failed to fetch contest info (url={})", contest_url))?;
    {
        let spinner = progress_spinner.lock().await;
        spinner.set_prefix("✅");
        spinner.finish_with_message("Fetching contest info ... Done");
    }

    log::info!("Creating each problem workspace");
    let bars_container = MultiProgress::new();
    let progress_header = bars_container
        .add(ProgressBar::new(1))
        .with_style(
            ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg:.magenta.bold}")
                .unwrap(),
        )
        .with_ticking();
    let progress_bar = bars_container
        .add(ProgressBar::new(contest.problems.len() as u64 * 3))
        .with_style(ProgressStyle::with_template("{prefix:.bold.dim} {wide_bar}").unwrap());

    let w = repo.workspace_home();
    let mut workspace_locations = Vec::new();

    let serial_code = style::contest_problem_serial_code_generator(contest.problems.len());

    for problem in &contest.problems {
        let problem_id = cli.extract_problem_id(&problem.url).unwrap();
        {
            let header = progress_header.lock().await;
            let prefix = format!("[{}/{}]", problem.ord, contest.problems.len());
            progress_bar.set_prefix(prefix.clone());
            header.set_prefix(prefix);
            header.set_message(format!("Fetching {}", problem_id));
        }

        // Avoid Dos attack
        std::thread::sleep(Duration::from_millis(300));
        progress_bar.inc(1);
        std::thread::sleep(Duration::from_millis(300));

        let (vault_loc, info) = self::ensure_problem_data_saved(cli, &problem.url, repo).await?;
        progress_bar.inc(1);
        progress_header
            .lock()
            .await
            .set_message(format!("Creating workspace for {}", problem_id));

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
        workspace_locations.push((loc, info));
        progress_bar.inc(1);
    }
    progress_bar.finish_and_clear();
    progress_header.lock().await.finish_and_clear();
    Ok(workspace_locations)
}

pub async fn do_test_with_runner(
    runner: &TestRunner,
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

    if cfg.compile_before_run && runner.is_compile_cmd_defined() {
        let cmd = runner.get_command().compile.as_ref().unwrap();
        log::info!("Compile: {}", cmd);
        runner.compile().await?;
    }

    let style = ProgressStyle::default_spinner();

    let mut results = Vec::with_capacity(testcases.len());
    let mut bars = Vec::with_capacity(testcases.len());
    let progress_bar_container = MultiProgress::new();

    log::info!("Running: {}", runner.get_command().run);

    // Prepare progress bar
    for t in &testcases {
        let bar = progress_bar_container
            .add(ProgressBar::new(100))
            .with_style(style.clone())
            .with_message(format!("Testcase {} ...", t.name()))
            .with_ticking();
        bars.push(bar);
    }

    for (t, bar) in testcases.iter().zip(&bars) {
        let res = runner
            .run(
                t,
                cfg.stdout_capture_max_bytes,
                cfg.stderr_capture_max_bytes,
            )
            .await?;
        bar.lock().await.finish_with_message({
            format!(
                "Testcase {} ... {}{} [{}ms]",
                t.name(),
                style::judge_icon(res.judge),
                " ".repeat(3 - res.judge.to_string().len()),
                res.execution_time.as_millis(),
            )
            .cyan()
            .to_string()
        });
        results.push(res);
    }
    print!("\n");

    results
        .iter()
        .filter(|x| x.judge != JudgeCode::AC)
        .for_each(style::print_test_result_detail);

    style::print_test_result_summary(&results);
    Ok(results)
}

pub async fn do_test_with_command(
    cmd: TestCommand,
    testcase_dir: impl AsRef<Path>,
    cfg: &TestConfig,
) -> Result<Vec<TestOutcome>> {
    let runner = TestRunner::new(cmd).shell(cfg.shell.to_owned());
    self::do_test_with_runner(&runner, testcase_dir, cfg).await
}

pub async fn do_test(
    program_file: impl AsRef<Path>,
    testcase_dir: impl AsRef<Path>,
    cfg: &TestConfig,
) -> Result<Vec<TestOutcome>> {
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

    self::do_test_with_runner(&runner, testcase_dir, cfg).await
}

pub async fn submit(
    cli: &SessionPersistentClient,
    program_file: impl AsRef<Path>,
    problem_url: &Url,
    cfg: &SubmissionConfig,
    available_langs: &[PgLang],
) -> Result<Url> {
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

    let submission_status_url = cli
        .submit(problem_url, &lang, &source_code)
        .await
        .with_context(|| {
            format!(
                "Failed to submit {:?} with specifying lang='{}' (langID={})",
                program_file.as_ref(),
                lang.name,
                lang.id
            )
        })?;

    Ok(submission_status_url)
}
