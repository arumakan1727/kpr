use clap::Parser;
use kyopro_cli::cmd::GlobalArgs;

#[tokio::main]
async fn main() {
    let app = GlobalArgs::parse();
    app.exec_subcmd().await;
}
