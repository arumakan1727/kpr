use std::fs;
use std::process::exit;
use strum::IntoEnumIterator;

use crate::client::new_client;
use crate::util;
use crate::{config::Config, Platform};

use super::GlobalArgs;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platforms: Vec<Platform>,

    #[arg(short, long)]
    pub all: bool,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> ! {
    let cfg = Config::from_file_and_args_or_die(global_args);

    if !args.all && args.platforms.is_empty() {
        eprintln!("Please specify platform in argument (or you can use '--all')");
        exit(1);
    }

    let platforms = {
        if args.all {
            Platform::iter().collect()
        } else {
            util::dedup(args.platforms.clone())
        }
    };

    for &platform in &platforms {
        println!("Trying to logout from {}...", platform);
        let mut cli = new_client(platform, &cfg);
        if !cli.is_logged_in() {
            eprintln!("Not logged in to {:?}", platform);
            continue;
        }
        let _ = cli.logout().await;
        let filepath = cfg.session_json_path(cli.platform_name());
        let _ = fs::remove_file(filepath);
        eprintln!("Successful logout from {}", platform);
    }

    exit(0)
}
