use clap::ValueEnum as _;
use kpr_core::{action, client::SessionPersistentClient};
use kpr_webclient::Platform;
use std::process::exit;

use super::{ArgPlatform, GlobalArgs};
use crate::{config::Config, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platforms: Vec<ArgPlatform>,

    #[arg(short, long)]
    pub all: bool,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> ! {
    if !args.all && args.platforms.is_empty() {
        eprintln!("Please specify platform in argument (or you can use '--all')");
        exit(1);
    }

    let platforms: Vec<ArgPlatform> = if args.all {
        ArgPlatform::value_variants().to_vec()
    } else {
        util::dedup(args.platforms.clone())
    };

    let cfg = Config::from_file_and_args(global_args);

    for platform in platforms.into_iter().map(Into::<Platform>::into) {
        let mut cli = SessionPersistentClient::new(platform, &cfg.cache_dir);
        action::logout(&mut cli).await.unwrap_or_else(|e| {
            eprintln!("{}", e);
            exit(1);
        });
        eprintln!("Successfully logged out from {}", platform);
    }
    exit(0)
}
