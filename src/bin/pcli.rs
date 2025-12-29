// cargo imports
use clap::{Parser, Subcommand};


// local imports
use pcli::modules::hardware;

#[derive(Parser)]
#[command(name = "pcli")]
#[command(about = "CLI client for system stats daemon and more for Dfltplyr :D")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get hardware information
    Hardware,
    /// QS actions
    Qs { action: String },
    /// Compositor commands
    Compositor { action: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hardware => {
            hardware::get_hardware_info()?;
        }
        Commands::Qs { action } => {
            println!("QS action requested: {}", action);
        }
        Commands::Compositor { action } => {
            println!("Compositor action requested: {}", action);
        }
    }
    Ok(())
}
