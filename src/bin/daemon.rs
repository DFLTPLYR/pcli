// cargo imports
use niri_ipc::{Response, socket::Socket};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
};

// local imports
use pcli::{DesktopEnvironment, modules::hardware};

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
        match request.trim() {
            "hardware_info" => {
                let _ = hardware::get_hardware_info(stream);
            }
            "compositor_data" => {
                match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => {
                        let _ = niri_ipc_listener(stream);
                    }
                    DesktopEnvironment::Unknown => { /* handle others */ }
                }
            }
            other => {
                println!("Unknown request: {}", other);
            }
        }
    }
}

fn niri_ipc_listener(mut stream: UnixStream) -> std::io::Result<()> {
    let mut socket = Socket::connect()?;

    let reply = socket.send(niri_ipc::Request::EventStream)?;
    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = socket.read_events();
        while let Ok(event) = read_event() {
            writeln!(stream, "{}", serde_json::to_string(&event).unwrap())?;
        }
    }

    Ok(())
}
