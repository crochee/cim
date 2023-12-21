use clap::{Parser, Subcommand};

use cimc::version;

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
    dotenv::dotenv().ok();
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
