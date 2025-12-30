use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

// cargo imports
use clap::Parser;

// local imports
use pcli::Commands;
use pcli::modules::shell;

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
            send_request("hardware_info".to_string())?;
            // daemon_command::get_hardware_info()?;
        }
        Commands::Launch { target } => {
            shell::shell_query(target);
        }
    }
    Ok(())
}

pub fn send_request(req: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect("/tmp/sysinfo.sock")?;
    writeln!(stream, "{}", req)?;
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}
