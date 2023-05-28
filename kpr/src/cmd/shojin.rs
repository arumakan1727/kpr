use kpr_core::{action, client::SessionPersistentClient, config::QualifiedRepoConfig};
use kpr_webclient::{detect_platform_from_url, Url};
use std::process::exit;

use super::GlobalArgs;
use crate::{config::Config, util};

#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg()] // positional argument
    pub problem_url: String,
}

const PROBLEM_URL_EXAMPLE: &str = "https://atcoder.jp/contests/abc001/tasks/abc001_1";

fn print_valid_problem_url_example() {
    eprintln!("Valid problem URL example: '{}'", PROBLEM_URL_EXAMPLE);
}

pub async fn exec(args: &Args, global_args: &GlobalArgs) -> ! {
    let Ok(url) = Url::parse(&args.problem_url) else {
        eprintln!("Cannot parse as URL: '{}'", args.problem_url);
        print_valid_problem_url_example();
        exit(1);
    };
    let Some(platform) = detect_platform_from_url(&url) else {
        eprintln!("Cannot detect platform from URL '{}'", args.problem_url);
        print_valid_problem_url_example();
        exit(1);
    };

    let cfg = Config::from_file_and_args(global_args);
    let cli = SessionPersistentClient::new(platform, &cfg.cache_dir);
    if !cli.is_problem_url(&url) {
        eprintln!("Not a problem URL: '{}'", args.problem_url);
        print_valid_problem_url_example();
        exit(1);
    }

    let repo = QualifiedRepoConfig::from_fs(util::current_dir()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(1);
    });

    let saved_loc = action::create_shojin_workspace(&cli, &url, &repo, &chrono::Local::now())
        .await
        .unwrap_or_else(|e| {
            eprintln!("{}: {}", e, e.source().unwrap());
            exit(1);
        });

    println!(
        "Successfully created shojin workspace in {}",
        saved_loc.dirpath().to_string_lossy()
    );
    exit(0)
}
