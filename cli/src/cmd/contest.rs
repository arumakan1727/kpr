use chrono::Local;
use kpr_core::{
    action, client::SessionPersistentClient, print_success, storage::Repository, style,
};

use super::{GlobalArgs, SubcmdResult};
use crate::{config::GlobalConfig, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub contest_url: String,
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> SubcmdResult {
    let cfg = GlobalConfig::from_file_and_args(global_args);
    let (cli, url) =
        SessionPersistentClient::new_with_parse_url(&args.contest_url, &cfg.cache_dir)?;

    let repo: Repository =
        kpr_core::Config::from_file_finding_in_ancestors(util::current_dir())?.into();

    let res = action::create_contest_workspace(&cli, &url, &repo, Local::now()).await?;
    let saved_dir = res[0].0.dir().parent().unwrap();
    let saved_dir = fsutil::relative_path(util::current_dir(), saved_dir);

    print_success!(
        "Successfully created {} workspaces in {:?} ✨",
        res.len(),
        saved_dir,
    );

    let serial_code = style::contest_problem_serial_code_generator(res.len());
    for (i, (_, info)) in res.iter().enumerate() {
        println!(" [{}] {}", serial_code(1 + i as u32), info.problem_id);
    }

    Ok(())
}
