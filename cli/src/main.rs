use clap::Parser;
use kpr_cli::cmd::GlobalArgs;

#[tokio::main]
async fn main() {
    let app = GlobalArgs::parse();
    app.exec_subcmd().await.unwrap_or_else(|e| {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    });
}
