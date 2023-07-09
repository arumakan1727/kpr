use std::path::PathBuf;

use kpr_core::action;

use super::{GlobalArgs, SubcmdResult};
use crate::util;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub filepath: PathBuf,

    #[arg(short, long)]
    pub out: Option<PathBuf>,
}

pub fn exec(args: &Args, _: &GlobalArgs) -> SubcmdResult {
    let cfg = kpr_core::Config::from_file_finding_in_ancestors(util::current_dir())?;
    let code = action::expand_source_code(&args.filepath, &cfg.expander)?;

    if let Some(out_path) = &args.out {
        fsutil::write(out_path, code)?;
    } else {
        print!("{}", code);
    }

    Ok(())
}
