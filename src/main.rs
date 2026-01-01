mod cli;
mod m3u;
mod output;
mod parser;
mod scanner;
mod types;

use clap::Parser;
use cli::Cli;
use output::Output;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let output = Output::new(cli.quiet, cli.verbose);
    output.info(&format!("Scanning {}...", cli.target.display()));
}
