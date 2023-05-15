use crate::{
    client::{AtCoderClient, Client},
    Platform,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platform: Platform,
}

pub fn exec(args: &Args) {
    let cli: Box<dyn Client> = match args.platform {
        Platform::AtCoder => Box::new(AtCoderClient::new()),
    };
}
