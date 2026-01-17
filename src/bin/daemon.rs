// cargo imports
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::net::{UnixListener, UnixStream},
    thread,
};

// local imports
use pcli::{DesktopEnvironment, modules::hardware, modules::wm};

fn main() {
    let socket_path = "/tmp/pdaemon.sock";
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path).unwrap();
    for stream in listener.incoming().flatten() {
        thread::spawn(move || {
            handle_client(stream);
        });
    }
}

fn handle_client(stream: UnixStream) {
    let mut reader = BufReader::new(&stream);
    let mut request = String::new();
    if reader.read_line(&mut request).is_ok() {
        let parts: Vec<&str> = request.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["hardware_info"] => {
                let _ = hardware::get_hardware_info(stream);
            }
            ["compositor_data"] => {
                match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => {
                        let _ = wm::niri_ipc_listener(stream);
                    }
                    DesktopEnvironment::Unknown => { /* handle others */ }
                }
            }
            ["generate_palette", targets @ ..] => {
                let targets: Vec<String> = targets.iter().map(|s| s.to_string()).collect();
                println!("Targets: {:?}", targets);
                let _ = stream;
            }
            [other, ..] => {
                println!("Unknown request: {}", other);
            }
            _ => {
                println!("Invalid request");
            }
        }
    }
}
