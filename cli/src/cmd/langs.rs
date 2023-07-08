use std::io;

use kpr_core::{action, client::SessionPersistentClient, storage::Repository};

use super::{ArgPlatform, GlobalArgs, SubcmdResult};
use crate::{config::GlobalConfig, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platform: ArgPlatform,

    #[arg(short, long)]
    pub json: bool,

    #[arg(short = 'N', long)]
    pub no_cache: bool,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = GlobalConfig::from_file_and_args(global_args);
    let cli = SessionPersistentClient::new(args.platform.into(), &cfg.cache_dir);
    let repo = Repository::from_config_file_finding_in_ancestors(util::current_dir())?;

    let (_, langs) = if args.no_cache {
        action::fetch_and_save_submittable_lang_list(&cli, &repo).await?
    } else {
        action::ensure_submittable_lang_list_saved(&cli, &repo).await?
    };

    if args.json {
        serde_json::to_writer_pretty(io::stdout(), &langs)?;
        return Ok(());
    }

    for lang in langs {
        println!("{}", lang.name);
    }
    Ok(())
}
