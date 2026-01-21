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
    DesktopEnvironment,
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
    let mut request = String::new();

    if reader.read_line(&mut request).is_ok() {
        let parts: Vec<&str> = request.trim().split_whitespace().collect();

        match parts.as_slice() {
            ["hardware_info"] => {
                hardware::get_hardware_info(stream);
            }
            ["compositor_data"] => match DesktopEnvironment::from_env() {
                DesktopEnvironment::Niri => {
                    wm::niri_ipc_listener(stream);
                }
                DesktopEnvironment::Unknown => {}
            },
            ["generate_palette", rest @ ..] => {
                let rest_vec: Vec<String> = rest.iter().map(|s| s.to_string()).collect();
                let type_ = rest_vec[0].clone();
                let paths = rest_vec[1..].to_vec();
                wallpaper::generate_color_palette(type_, paths, stream);
            }
            ["window_manager_rules"] => match DesktopEnvironment::from_env() {
                DesktopEnvironment::Niri => {
                    wm::get_rules(stream);
                }
                DesktopEnvironment::Unknown => {}
            },
            ["weather"] | ["weather_watcher"] => {
                weather::get_weather_info(stream, None);
            }
            _ => {
                println!("Unknown request");
            }
        }

        // 🔴 IMPORTANT: keep socket alive briefly
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
