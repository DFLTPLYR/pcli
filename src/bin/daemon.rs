// cargo imports

use std::{
    env,
    fs::{self, Permissions},
    io::{self, BufRead, BufReader, ErrorKind},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
};
// local imports
use pcli::{
    modules::{hardware, wallpaper, weather, wm},
    DesktopEnvironment, Request,
};

fn main() -> io::Result<()> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");

    let socket_path = format!("{}/pdaemon.sock", runtime_dir);

    // Remove stale socket (if any)
    if let Err(e) = fs::remove_file(&socket_path) {
        if e.kind() != ErrorKind::NotFound {
            return Err(e);
        }
    }

    let listener = UnixListener::bind(&socket_path)?;

    // Explicit permissions (defensive, but correct)
    fs::set_permissions(&socket_path, Permissions::from_mode(0o600))?;

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Set non-blocking so we can check the flag
    listener.set_nonblocking(true)?;

    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let running_clone = running.clone();
                thread::spawn(move || {
                    handle_client(stream, running_clone);
                });
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                // No incoming connections, sleep briefly to avoid busy loop
                thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("accept error: {e}");
            }
        }
    }

    // Clean up socket file on exit
    let _ = fs::remove_file(&socket_path);

    Ok(())
}

fn handle_client(stream: UnixStream, running: Arc<AtomicBool>) {
    let mut reader = BufReader::new(&stream);
    let mut request_str = String::new();

    if reader.read_line(&mut request_str).is_ok() {
        if let Some(request) = Request::from_string(&request_str) {
            match request {
                Request::HardwareInfo => {
                    hardware::get_hardware_info(stream, running);
                }
                Request::CompositorData => match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => wm::niri_ipc_listener(stream, running),
                    DesktopEnvironment::Hyprland => wm::hyprland_ipc_listener(stream, running),
                    DesktopEnvironment::Unknown => {}
                },
                Request::GeneratePalette { type_, paths } => {
                    wallpaper::generate_color_palette(type_, paths, stream);
                }
                Request::WindowManagerRules => match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => wm::get_rules(stream),
                    DesktopEnvironment::Hyprland => {}
                    DesktopEnvironment::Unknown => {}
                },
                Request::Weather { use_curl } => {
                    let use_curl = use_curl.unwrap_or(true);
                    weather::get_weather_info(stream, use_curl, running);
                }
            }
        } else {
            println!("Unknown request: {}", request_str.trim());
        }
    }
}
