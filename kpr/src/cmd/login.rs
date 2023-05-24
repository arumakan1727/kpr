use kpr_core::{action, client::SessionPersistentClient};
use std::process::exit;

use super::{ArgPlatform, GlobalArgs};
use crate::config::Config;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platform: ArgPlatform,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> ! {
    let platform = args.platform.into();
    let cfg = Config::from_file_and_args_or_die(global_args);

    let mut cli = SessionPersistentClient::new(platform, &cfg.cache_dir);

    action::login(&mut cli).await.unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1);
    });

    println!("Successfully logged in to {}", platform);
    exit(0)
}
