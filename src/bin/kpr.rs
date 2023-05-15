use clap::Parser;
use kyopro_cli::subcmd::Subcommand;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Subcommand,
}

fn main() {
    let args = Cli::parse();
    args.cmd.exec();
}
