use kpr_core::{action, config::RepoConfig};
use std::{path::PathBuf, process::exit};

use crate::util;

use super::GlobalArgs;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(default_value = "./")]
    dir: PathBuf,
}

pub fn exec(args: &Args, _: &GlobalArgs) -> ! {
    if let Some(repo_config_path) = RepoConfig::find_filepath(util::current_dir()) {
        eprintln!(
            "Current dir is already in kpr-repository. (config path: '{}')",
            repo_config_path.to_string_lossy()
        );
        eprintln!("Canceled initializing.");
        exit(1);
    }

    action::init_kpr_repository(&args.dir).unwrap_or_else(|e| {
        eprintln!("Failed to init kpr repository: {}", e);
        exit(1);
    });
    println!(
        "Successfully initialized kpr repository. (path: {})",
        args.dir.to_string_lossy()
    );
    exit(0)
}
