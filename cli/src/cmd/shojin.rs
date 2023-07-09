use chrono::Local;
use kpr_core::{action, client::SessionPersistentClient, print_success, storage::Repository};

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

    let repo: Repository =
        kpr_core::Config::from_file_finding_in_ancestors(util::current_dir())?.into();

    let saved_loc = action::create_shojin_workspace(&cli, &url, &repo, Local::now()).await?;
    let saved_dir = fsutil::relative_path(util::current_dir(), saved_loc.dir());

    print_success!(
        "Successfully created shojin workspace in {:?} ✨",
        saved_dir,
    );
    Ok(())
}
