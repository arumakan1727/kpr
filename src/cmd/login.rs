use std::fs;
use std::process::exit;

use crate::client::new_client;
use crate::{config::Config, Platform};

use super::GlobalArgs;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platform: Platform,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> ! {
    let cfg = Config::from_file_and_args_or_die(global_args);

    let mut cli = new_client(args.platform, &cfg);
    if cli.is_logged_in() {
        eprintln!("Already logged in to {}", args.platform);
        exit(1);
    }

    let cred = cli.ask_credential().unwrap_or_else(|e| {
        eprintln!("{:#}", e);
        exit(1);
    });
    cli.login(cred).await.unwrap_or_else(|e| {
        eprintln!("Failed to login: {:#}", e);
        exit(1);
    });

    let save_path = cfg.session_json_path(cli.platform_name());
    let res = fs::create_dir_all(save_path.parent().unwrap()).and_then(|_| {
        let json = cli.auth_data().to_json();
        fs::write(save_path, json)
    });
    match res {
        Ok(()) => {
            eprintln!("Successful login to {}", args.platform);
            exit(0);
        }
        Err(e) => {
            eprintln!("Failed to save login session: {}", e);
            exit(1);
        }
    }
}
