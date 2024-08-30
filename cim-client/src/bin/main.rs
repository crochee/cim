use clap::{Parser, Subcommand};

use cim_client::version;

// A fictional versioning CLI
#[derive(Debug, Parser)]
#[command(name = "cimctl")]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run,
    #[command(short_flag = 'v')]
    Version,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Run => {
            println!("run {}", 12);
        }
        Commands::Version => {
            println!("{}", version());
        }
    }
}
