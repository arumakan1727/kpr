use kpr_core::{action, config::Config, print_success};
use std::path::PathBuf;

use super::{GlobalArgs, SubcmdResult};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(default_value = "./")]
    dir: PathBuf,

    #[arg(short = 'P', long)]
    just_print: bool,
}

pub fn exec(args: &Args, _: &GlobalArgs) -> SubcmdResult {
    if args.just_print {
        println!("{}", Config::example_toml());
        return Ok(());
    }

    action::init_kpr_repository(&args.dir)?;
    print_success!(
        "Successfully initialized kpr repository. (path: {})",
        args.dir.to_string_lossy()
    );
    Ok(())
}
