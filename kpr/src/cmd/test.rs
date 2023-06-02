use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use kpr_core::{action, config::Config, fsutil, storage::ProblemWorkspace};

use crate::util;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub program_file_or_workspace_dir: Option<PathBuf>,

    #[arg(short = 'd', long)]
    pub testcase_dir: Option<PathBuf>,
}

pub async fn exec(args: &Args, _global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = Config::from_file_finding_in_ancestors(util::current_dir())?;

    let program_file = {
        let existing_path = match &args.program_file_or_workspace_dir {
            Some(path) if path.exists() => path,
            Some(path) => bail!("No such file or dir: {:?}", path),
            None => Path::new("./"),
        };

        if existing_path.is_dir() {
            fsutil::find_most_recently_modified_file(&existing_path, &cfg.test.include)
                .with_context(|| {
                    format!("Cannot find target program file in {:?}", existing_path)
                })?
        } else {
            existing_path.into()
        }
    };

    let testcase_dir = args
        .testcase_dir
        .clone()
        .unwrap_or_else(|| ProblemWorkspace::new(Path::new(".")).testcase_dir());

    let _ = action::do_test(program_file, testcase_dir, &cfg.test).await?;
    Ok(())
}
