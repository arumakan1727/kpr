use std::path::{Path, PathBuf};

use kpr_core::{action, config::Config, storage::ProblemWorkspace, testing::TestCommand};

use crate::util;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub program_file_or_workspace_dir: Option<PathBuf>,

    #[arg(short = 'd', long)]
    pub testcase_dir: Option<PathBuf>,

    #[arg(short, long)]
    pub cmd: Option<String>,
}

pub async fn exec(args: &Args, _global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = Config::from_file_finding_in_ancestors(util::current_dir())?;

    let testcase_dir = args
        .testcase_dir
        .clone()
        .unwrap_or_else(|| ProblemWorkspace::new(Path::new(".")).testcase_dir());

    if let Some(run_cmd) = args.cmd.as_ref() {
        let cmd = TestCommand {
            compile: None,
            run: run_cmd.to_owned(),
        };
        action::do_test_with_command(cmd, testcase_dir, &cfg.test).await?;
        return Ok(());
    }

    let program_file =
        util::determine_program_file(&args.program_file_or_workspace_dir, &cfg.test.include)?;

    let _ = action::do_test(program_file, testcase_dir, &cfg.test).await?;

    Ok(())
}
