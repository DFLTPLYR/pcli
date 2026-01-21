// cargo imports
use clap::Parser;
use std::{
    fmt::Display,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

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
        }
        Commands::Compositor => {
            send_request("compositor_data".to_string())?;
        }
        Commands::Launch { target } => {
            shell::shell_query(target);
        }
        Commands::GeneratePalette { type_, paths } => {
            let args = format!("{} {}", type_, paths.join(" "));
            send_request_with_opt("generate_palette".to_string(), Some(args))?;
        }
        Commands::Rules => {
            send_request("window_manager_rules".to_string())?;
        }
        Commands::Weather => {
            send_request("weather_watcher".to_string())?;
        }
    }
    Ok(())
}

pub fn send_request(req: String) -> Result<(), Box<dyn std::error::Error>> {
    send_request_with_opt(req, None::<String>)
}

pub fn send_request_with_opt<T: Display>(
    req: String,
    opt: Option<T>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect("/tmp/pdaemon.sock")?;
    if let Some(o) = opt {
        writeln!(stream, "{} {}", req, o)?;
    } else {
        writeln!(stream, "{}", req)?;
    }
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }
    Ok(())
}
