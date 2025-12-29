use clap::{Parser, Subcommand};
use std::{
    io::{BufRead, BufReader},
    os::unix::net::UnixStream,
};
use serde_json::from_str;
use pcli::MySysInfo;

#[derive(Parser)]
#[command(name = "pcli")]
#[command(about = "CLI client for system stats daemon")]
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hardware => {
            let stream = UnixStream::connect("/tmp/sysinfo.sock")
                .expect("Daemon not running");
            let reader = BufReader::new(stream);

            for line in reader.lines() {
                let line = line.unwrap();
                let info: MySysInfo = from_str(&line).unwrap();
                println!("Memory: {} / {}", info.used_memory, info.total_memory);
            }
        }
        Commands::Qs { action } => {
            println!("QS action requested: {}", action);
        }
        Commands::Compositor { action } => {
            println!("Compositor action requested: {}", action);
        }
    }
}

