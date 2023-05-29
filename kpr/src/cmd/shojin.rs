use chrono::Local;
use kpr_core::{action, client::SessionPersistentClient, storage::Repository};

use super::{GlobalArgs, SubcmdResult};
use crate::{config::Config, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub problem_url: String,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = Config::from_file_and_args(global_args);
    let (cli, url) =
        SessionPersistentClient::new_with_parse_url(&args.problem_url, &cfg.cache_dir)?;

    let repo = Repository::from_config_file_finding_in_ancestors(util::current_dir())?;

    let saved_loc = action::create_shojin_workspace(&cli, &url, &repo, Local::now()).await?;

    println!(
        "Successfully created shojin workspace in {}",
        saved_loc.dirpath().to_string_lossy()
    );
    Ok(())
}
