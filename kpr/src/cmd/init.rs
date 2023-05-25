use kpr_core::action;
use std::{path::PathBuf, process::exit};

use super::GlobalArgs;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(default_value = "./")]
    dir: PathBuf,
}

pub fn exec(args: &Args, _: &GlobalArgs) -> ! {
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
