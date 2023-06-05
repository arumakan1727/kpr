use std::io::Write;

use clap::Parser;
use colored::Colorize;
use kpr_cli::cmd::GlobalArgs;
use kpr_core::color::{DefaultPalette, SemanticColor};
use log::LevelFilter;

#[tokio::main]
async fn main() {
    init_logger();

    let app = GlobalArgs::parse();
    app.exec_subcmd().await.unwrap_or_else(|e| {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    });
}

fn init_logger() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format(|f, r| {
            let c = DefaultPalette.level(r.level());
            let msg = r.args();
            let a = msg
                .as_str()
                .map(|s| s.color(c))
                .unwrap_or_else(|| msg.to_string().color(c));
            let s = format!(
                "[{}] {}: {}",
                r.level().as_str().bold().color(c),
                r.target(),
                a,
            );
            writeln!(f, "{}", s)
        })
        .parse_default_env()
        .init();
}
