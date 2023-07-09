use std::path::{Path, PathBuf};

use anyhow::{ensure, Context as _};
use colored::Colorize;
use kpr_core::{
    action::{self, ensure_submittable_lang_list_saved},
    client::SessionPersistentClient,
    config::Config,
    storage::{ProblemWorkspace, Repository},
    testing::JudgeCode,
};

use crate::{config::GlobalConfig, util};

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub program_file_or_workspace_dir: Option<PathBuf>,

    #[arg(short, long)]
    pub test: bool,

    #[arg(short = 'N', long)]
    pub no_test: bool,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    ensure!(
        !(args.test && args.no_test),
        "Conflict option: '--test' and '--no-test'"
    );

    let cfg = Config::from_file_finding_in_ancestors(util::current_dir())?;
    let global_cfg = GlobalConfig::from_file_and_args(&global_args);

    let program_file =
        util::determine_program_file(&args.program_file_or_workspace_dir, &cfg.test.include)?;

    let workspace = ProblemWorkspace::new(Path::new("."));

    // デフォルト値は設定ファイルの値、それをコマンドラインオプションで上書き
    // (論理式は簡略化済み)
    let run_test = args.test | (cfg.submit.run_test & !args.no_test);
    if run_test {
        let res = action::do_test(&program_file, workspace.testcase_dir(), &cfg.test).await?;
        if res.iter().any(|x| x.judge != JudgeCode::AC) {
            println!(
                "{}",
                "Canceling submission due to test failure.".bright_red()
            );
            return Ok(());
        }
    }

    let (problem_url, platform) = {
        let info = workspace
            .load_problem_info()
            .context("Failed to get problem URL")?;
        (info.url, info.platform)
    };

    let cli = SessionPersistentClient::new(platform, &global_cfg.cache_dir);
    let repo = Repository::from(cfg.clone());

    let (_, available_langs) = ensure_submittable_lang_list_saved(&cli, &repo).await?;

    let submission_status_url =
        action::submit(&cli, &program_file, &problem_url, &cfg, &available_langs).await?;

    println!(
        "{}\nSubmission status URL:\n  {}",
        format!(
            "Successfully submitted {:?} to {}",
            program_file, problem_url
        )
        .green(),
        submission_status_url.to_string().cyan(),
    );

    Ok(())
}
