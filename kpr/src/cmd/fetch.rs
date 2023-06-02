use kpr_core::{action, client::SessionPersistentClient, storage::Repository};

use super::{GlobalArgs, SubcmdResult};
use crate::{config::GlobalConfig, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub problem_url: String,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = GlobalConfig::from_file_and_args(global_args);
    let (cli, url) =
        SessionPersistentClient::new_with_parse_url(&args.problem_url, &cfg.cache_dir)?;

    let repo = Repository::from_config_file_finding_in_ancestors(util::current_dir())?;

    let (problem_dir, _, _) = action::fetch_and_save_problem_data(&cli, &url, &repo).await?;

    println!(
        "Successfully saved problem data in '{}'",
        problem_dir.dir().to_string_lossy()
    );
    Ok(())
}
