// cargo imports
use clap::Parser;

// local imports
use pcli::Commands;
use pcli::modules::{hardware, shell};

#[derive(Parser)]
#[command(name = "pcli")]
#[command(about = "CLI client for system stats daemon and more for Dfltplyr :D")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hardware => {
            hardware::get_hardware_info()?;
        }
        Commands::Launch { target } => {
            shell::shellquery(target);
        }
    }
    Ok(())
}
