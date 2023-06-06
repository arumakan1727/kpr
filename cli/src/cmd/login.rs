use kpr_core::{action, client::SessionPersistentClient, print_success};

use super::{ArgPlatform, GlobalArgs, SubcmdResult};
use crate::config::GlobalConfig;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platform: ArgPlatform,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    let platform = args.platform.into();
    let cfg = GlobalConfig::from_file_and_args(global_args);

    let mut cli = SessionPersistentClient::new(platform, &cfg.cache_dir);

    action::login(&mut cli).await?;
    print_success!("Successfully logged in to {}", platform);
    Ok(())
}
