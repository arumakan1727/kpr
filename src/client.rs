pub mod atcoder;
pub mod common;

use std::{fs, io::ErrorKind};

pub use atcoder::AtCoderClient;
pub use common::*;

use crate::{config::Config, Platform};

pub fn new_client(platform: Platform, cfg: &Config) -> Box<dyn Client> {
    use Platform::*;
    let mut cli: Box<dyn Client> = match platform {
        AtCoder => Box::new(AtCoderClient::new()),
    };

    let json_path = cfg.session_json_path(cli.platform_name());
    match fs::read_to_string(&json_path) {
        Ok(json) => {
            if let Err(e) = cli.set_auth_data_from_json(&json) {
                eprintln!(
                    "Error on initializing client: {}: {}",
                    json_path.to_string_lossy(),
                    e
                );
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => (),
            _ => panic!(
                "Error on initializing client: {}: {:?}",
                json_path.to_string_lossy(),
                e
            ),
        },
    };
    cli
}

#[cfg(test)]
mod atcoder_test;
