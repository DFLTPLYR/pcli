// cargo imports

use std::{
    env, fs,
    io::{self, BufRead, BufReader},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    thread,
};
// local imports
use pcli::{
    DesktopEnvironment, Request,
    modules::{hardware, wallpaper, weather, wm},
};

fn main() -> io::Result<()> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");

    let socket_path = format!("{}/pdaemon.sock", runtime_dir);

    // Remove stale socket (if any)
    if let Err(e) = fs::remove_file(&socket_path) {
        if e.kind() != io::ErrorKind::NotFound {
            return Err(e);
        }
    }

    let listener = UnixListener::bind(&socket_path)?;

    // Explicit permissions (defensive, but correct)
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }

    Ok(())
}

fn handle_client(stream: UnixStream) {
    let mut reader = BufReader::new(&stream);
    let mut request_str = String::new();

    if reader.read_line(&mut request_str).is_ok() {
        if let Some(request) = Request::from_string(&request_str) {
            match request {
                Request::HardwareInfo => {
                    hardware::get_hardware_info(stream);
                }
                Request::CompositorData => match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => wm::niri_ipc_listener(stream),
                    DesktopEnvironment::Unknown => {}
                },
                Request::GeneratePalette { type_, paths } => {
                    wallpaper::generate_color_palette(type_, paths, stream);
                }
                Request::WindowManagerRules => match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => wm::get_rules(stream),
                    DesktopEnvironment::Unknown => {}
                },
                Request::Weather | Request::WeatherWatcher => {
                    weather::get_weather_info(stream, None);
                }
            }
        } else {
            println!("Unknown request: {}", request_str.trim());
        }
    }
}
