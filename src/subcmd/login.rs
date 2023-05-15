use std::process::exit;

use crate::{
    client::{AtCoderClient, Client},
    Platform,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub platform: Platform,
}

pub async fn exec(args: &Args) {
    use Platform::*;
    let mut cli: Box<dyn Client> = match args.platform {
        AtCoder => Box::new(AtCoderClient::new()),
    };

    let cred = cli.ask_credential().unwrap_or_else(|e| {
        eprintln!("{:#}", e);
        exit(1);
    });

    match cli.login(cred).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {:#}", e);
            exit(1);
        }
    };
}
