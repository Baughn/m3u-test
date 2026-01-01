mod cli;
mod m3u;
mod parser;
mod types;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Target: {:?}", cli.target);
}
