pub mod contest;
pub mod fetch;
pub mod init;
pub mod login;
pub mod logout;
pub mod shojin;
pub mod test;

use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
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
    Init(init::Args),
    Fetch(fetch::Args),
    Shojin(shojin::Args),
    Contest(contest::Args),

    #[command(alias("t"))]
    Test(test::Args),
}

pub type SubcmdResult = anyhow::Result<()>;

impl GlobalArgs {
    pub async fn exec_subcmd(&self) -> SubcmdResult {
        use Subcommand::*;
        match &self.subcmd {
            Login(args) => login::exec(args, self).await,
            Logout(args) => logout::exec(args, self).await,
            Init(args) => init::exec(args, self),
            Fetch(args) => fetch::exec(args, self).await,
            Shojin(args) => shojin::exec(args, self).await,
            Contest(args) => contest::exec(args, self).await,
            Test(args) => test::exec(args, self).await,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum ArgPlatform {
    AtCoder,
}

impl From<ArgPlatform> for kpr_webclient::Platform {
    fn from(value: ArgPlatform) -> Self {
        use kpr_webclient::Platform;
        use ArgPlatform::*;
        match value {
            AtCoder => Platform::AtCoder,
        }
    }
}

impl From<&ArgPlatform> for kpr_webclient::Platform {
    fn from(&value: &ArgPlatform) -> Self {
        value.into()
    }
}
