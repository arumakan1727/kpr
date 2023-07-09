use crate::util;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn exec(_args: &Args, _global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = kpr_core::Config::from_file_finding_in_ancestors(util::current_dir())?;
    println!(
        "{}",
        cfg.source_config_dir.canonicalize()?.to_string_lossy()
    );
    Ok(())
}
