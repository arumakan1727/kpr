use std::path::PathBuf;

use clap::Parser;

pub mod login;
pub mod logout;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct GlobalArgs {
    #[command(subcommand)]
    pub subcmd: Subcommand,

    #[arg(long)]
    pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Login(login::Args),
    Logout(logout::Args),
}

impl GlobalArgs {
    pub async fn exec_subcmd(&self) -> () {
        use Subcommand::*;
        match &self.subcmd {
            Login(args) => login::exec(args, self).await,
            Logout(args) => logout::exec(args, self).await,
        }
    }
}
