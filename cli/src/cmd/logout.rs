use anyhow::ensure;
use clap::ValueEnum as _;
use kpr_core::{action, client::SessionPersistentClient};
use kpr_webclient::Platform;

use super::{ArgPlatform, GlobalArgs, SubcmdResult};
use crate::{config::GlobalConfig, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platforms: Vec<ArgPlatform>,

    #[arg(short, long)]
    pub all: bool,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    ensure!(
        !args.platforms.is_empty() || args.all,
        "Please specify <PLATFORM> in argument (or you can use '--all')"
    );

    let platforms: Vec<ArgPlatform> = if args.all {
        ArgPlatform::value_variants().to_vec()
    } else {
        util::dedup(args.platforms.clone())
    };

    let cfg = GlobalConfig::from_file_and_args(global_args);

    for platform in platforms.into_iter().map(Into::<Platform>::into) {
        let mut cli = SessionPersistentClient::new(platform, &cfg.cache_dir);
        action::logout(&mut cli).await?;
        println!("Successfully logged out from {}", platform);
    }
    Ok(())
}
