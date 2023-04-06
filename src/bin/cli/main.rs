use std::error::Error;

use clap::{Parser, Subcommand};

mod display;
mod list;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all files in the archive
    #[clap(visible_alias = "l", visible_alias = "ls")]
    List(list::Arguments),
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();
    match cli.command {
        Commands::List(arguments) => list::run(arguments),
    }
}
