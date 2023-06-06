use std::io::Write;

use clap::Parser;
use colored::Colorize;
use kpr_cli::cmd::GlobalArgs;
use kpr_core::style::ColorTheme as _;
use log::LevelFilter;

#[tokio::main]
async fn main() {
    init_logger();

    let app = GlobalArgs::parse();
    app.exec_subcmd().await.unwrap_or_else(|e| {
        eprintln!("{}: {}", "Error".red(), format!("{:?}", e).bright_red());
        std::process::exit(1);
    });
}

fn init_logger() {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .format(|f, r| {
            let c = r.level().color();
            let msg = r.args();
            let a = msg
                .as_str()
                .map(|s| s.color(c))
                .unwrap_or_else(|| msg.to_string().color(c));
            let s = format!("[{}] {}", r.level().as_str().bold().color(c), a);
            writeln!(f, "{}", s)
        })
        .parse_default_env()
        .init();
}
