// cargo imports
use clap::Parser;
use rfd::FileDialog;
use std::{
    env,
    error::Error,
    fmt::Display,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

// local imports
use pcli::modules::shell;
use pcli::Commands;

#[derive(Parser)]
#[command(name = "pcli")]
#[command(about = "CLI client for system stats daemon and more for Dfltplyr :D")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Hardware => {
            send_request("hardware".to_string())?;
        }
        Commands::Compositor => {
            send_request("compositor".to_string())?;
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
            send_request("weather".to_string())?;
        }
        Commands::FilePicker => {
            if let Some(file) = FileDialog::new().pick_file() {
                println!("{}", file.display());
            }
        }
    }
    Ok(())
}

pub fn send_request(req: String) -> Result<(), Box<dyn Error>> {
    send_request_with_opt(req, None::<String>)
}

pub fn send_request_with_opt<T: Display>(
    req: String,
    opt: Option<T>,
) -> Result<(), Box<dyn Error>> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");

    let socket_path = format!("{}/pdaemon.sock", runtime_dir);

    let mut stream = UnixStream::connect(socket_path)?;
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
