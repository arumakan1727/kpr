pub mod login;

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Login(login::Args),
}

impl Subcommand {
    pub fn exec(&self) {
        use Subcommand::*;
        match self {
            Login(args) => login::exec(args),
        }
    }
}
