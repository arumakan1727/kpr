use kpr_core::action;
use std::path::PathBuf;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(default_value = "./")]
    dir: PathBuf,
}

pub fn exec(args: &Args, _: &GlobalArgs) -> SubcmdResult {
    action::init_kpr_repository(&args.dir)?;
    println!(
        "Successfully initialized kpr repository. (path: {})",
        args.dir.to_string_lossy()
    );
    Ok(())
}
