use kpr_core::{action, print_success};
use std::path::PathBuf;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(default_value = "./")]
    dir: PathBuf,
}

pub fn exec(args: &Args, _: &GlobalArgs) -> SubcmdResult {
    action::init_kpr_repository(&args.dir)?;
    print_success!(
        "Successfully initialized kpr repository. (path: {})",
        args.dir.to_string_lossy()
    );
    Ok(())
}
