pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Local};
use colored::{Color, Colorize};
use crossterm::terminal;
use error::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kpr_webclient::problem_id::ProblemGlobalId;
use kpr_webclient::{PgLang, ProblemInfo, SampleTestcase, Url};
use tokio::sync::Mutex;

use crate::client::SessionPersistentClient;
use crate::config::{SubmissionConfig, TestConfig};
use crate::interactive::ask_credential;
use crate::storage::{
    workspace, PlatformVault, ProblemVault, ProblemWorkspace, Repository, WorkspaceNameModifier,
};
use crate::style;
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

        let (vault_loc, _info) = self::ensure_problem_data_saved(cli, &problem.url, repo).await?;
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
        log::info!("Compiling {}", filename);
        log::info!("{}", cmd);
        runner.compile().await?;
    }

    let style = ProgressStyle::default_bar()
        .template("{spinner} {msg}")
        .unwrap();

    let mut results = Vec::with_capacity(testcases.len());
    let mut bars = Vec::with_capacity(testcases.len());
    let progress_bar_container = MultiProgress::new();

    log::info!("Running: {}", runner.get_command().run);

    // Prepare progress bar
    for t in &testcases {
        let bar = progress_bar_container
            .add(ProgressBar::new(100))
            .with_style(style.clone())
            .with_message(format!("Testcase {} ...", t.name()));
        let bar = Arc::new(Mutex::new(bar));
        bars.push(bar.clone());

        // Tick spinner
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let bar = bar.lock().await;
                if bar.is_finished() {
                    break;
                }
                bar.tick();
            }
        });
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
        .for_each(print_test_result_detail);

    print_test_result_summary(&results);
    Ok(results)
}

fn print_test_result_summary(results: &[TestOutcome]) {
    let bar = "-".repeat(5);
    print!("{} ", bar);

    let count: HashMap<JudgeCode, usize> = results.iter().fold(HashMap::new(), |mut count, r| {
        *count.entry(r.judge).or_default() += 1;
        count
    });

    let num_total_test = results.len();
    let num_passed = *count.get(&JudgeCode::AC).unwrap_or(&0);
    let num_failed = num_total_test - num_passed;

    if num_passed == num_total_test {
        let msg = format!("All {} tests passed ✨", num_total_test);
        print!("{}", msg.green());
    } else {
        let summary_msg = if num_passed > 0 {
            format!("{}/{} tests failed 💣", num_failed, num_total_test)
        } else {
            format!("All {} tests failed 💀", num_total_test)
        };

        let detail_msg = count
            .iter()
            .filter(|(&judge, _)| judge != JudgeCode::AC)
            .map(|(&judge, &cnt)| {
                format!(
                    "{}{}{}",
                    style::judge_icon(judge),
                    "x".dimmed(),
                    cnt.to_string().bold().bright_white(),
                )
            })
            .collect::<Vec<String>>()
            .join(", ");

        print!("{} ({})", summary_msg.bright_red(), detail_msg);
    }

    println!(" {}", bar);
}

pub fn print_test_result_detail(res: &TestOutcome) {
    let stdout_lines: Vec<_> = res.output.stdout.lines().collect();
    let truth_lines: Vec<_> = res.groundtruth.lines().collect();

    let (cols, _) = terminal::size().unwrap_or((40, 40));

    const BOLD_LINE: &str = "━";
    const THIN_LINE: &str = "─";

    let bold_bar = BOLD_LINE.repeat(cols as usize).blue().bold();

    let title_color = Color::BrightYellow;
    println!(
        "\n{}: {} [{}ms]\n{}",
        res.testcase_name.color(title_color).bold(),
        style::judge_icon(res.judge),
        res.execution_time.as_millis(),
        bold_bar,
    );

    fn print_sub_title(s: &str, cols: usize) {
        println!(
            "{}{}",
            s.cyan().bold(),
            THIN_LINE.repeat(cols - s.len() - 1).bright_black(),
        )
    }

    fn print_lines(lines: &[&str], entire_str: &str) {
        if lines.is_empty() {
            println!("{}", "<EMPTY>".magenta().dimmed());
            return;
        }
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end();
            print!("{}", trimmed);

            let num_trailling_whitespace = line.len() - trimmed.len();
            if num_trailling_whitespace > 0 {
                print!(
                    "{}{}",
                    " ".repeat(num_trailling_whitespace).on_red(),
                    "(Trailling whitespace)".bright_red().bold()
                );
            }

            let is_last_line = i + 1 == lines.len();
            if is_last_line && !entire_str.ends_with("\n") {
                print!("{}", " Missing new line ".on_yellow().black().bold());
            }

            println!("");
        }
    }

    print_sub_title("[truth-answer]", cols as usize);
    print_lines(&truth_lines, &res.groundtruth);

    print_sub_title("[stdout]", cols as usize);
    print_lines(&stdout_lines, &res.output.stdout);

    if !res.output.stderr.is_empty() {
        print_sub_title("[stderr]", cols as usize);
        print!("{}", res.output.stderr);
    }

    println!("{}", bold_bar);
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
