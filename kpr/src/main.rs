use clap::Parser;
use kpr::cmd::GlobalArgs;

#[tokio::main]
async fn main() {
    let app = GlobalArgs::parse();
    app.exec_subcmd().await;
}
