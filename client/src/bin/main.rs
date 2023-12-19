use clap::{Parser, Subcommand};

use cimc::version;

// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "server")]
#[command(author, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Run,
    #[command(short_flag = 'v')]
    Version,
}

fn main() {
    dotenv::dotenv().ok();
    let args = Cli::parse();
    match args.command {
        Commands::Run => {
            println!("run");
        }
        Commands::Version => {
            println!("{}", version());
        }
    }
}
