use kpr_core::storage::Repository;

use crate::util;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn exec(_args: &Args, _global_args: &GlobalArgs) -> SubcmdResult {
    let repo = Repository::from_config_file_finding_in_ancestors(util::current_dir())?;
    println!("{}", repo.repo_root.canonicalize()?.to_string_lossy());
    Ok(())
}
