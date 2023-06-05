use chrono::Local;
use kpr_core::{action, client::SessionPersistentClient, storage::Repository};

use super::{GlobalArgs, SubcmdResult};
use crate::{config::GlobalConfig, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub contest_url: String,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = GlobalConfig::from_file_and_args(global_args);
    let (cli, url) =
        SessionPersistentClient::new_with_parse_url(&args.contest_url, &cfg.cache_dir)?;

    let repo = Repository::from_config_file_finding_in_ancestors(util::current_dir())?;

    let saved_locs = action::create_contest_workspace(&cli, &url, &repo, Local::now()).await?;

    println!("Successfully created {} workspaces.", saved_locs.len());
    Ok(())
}
