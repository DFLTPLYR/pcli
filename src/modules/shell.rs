use niri_ipc::Request;
use niri_ipc::socket::Socket;

use crate::LaunchTarget;
use std::process::Command;

enum DesktopEnvironment {
    Niri,
    Unknown,
}

impl DesktopEnvironment {
    fn from_env() -> Self {
        match std::env::var("XDG_CURRENT_DESKTOP") {
            Ok(val) if val == "Niri" => DesktopEnvironment::Niri,
            _ => DesktopEnvironment::Unknown,
        }
    }
}

pub fn shell_query(target: &LaunchTarget) {
    match target {
        LaunchTarget::WallpaperPicker => wallpaper_picker(),
        LaunchTarget::AppLauncher => println!("Quickshell: Launching App Launcher"),
        LaunchTarget::ExtendedBar => println!("Quickshell: Launching Extended Bar"),
    }
}

fn wallpaper_picker() {
    match DesktopEnvironment::from_env() {
        DesktopEnvironment::Niri => {
            let mut socket = Socket::connect().expect("Failed to connect to Niri socket");
            let reply = socket.send(Request::FocusedWindow);
            let qs = is_qs_running();
            if qs {
                println!("{:?}", reply);
                qs_ipc_caller(
                    &"~/dotfiles/.config/niri/shell".to_string(),
                    &"toggleWallpaperPicker".to_string(),
                );
            }
        }
        DesktopEnvironment::Unknown => { /* handle others */ }
    }
}

fn is_qs_running() -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg("qs")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn qs_ipc_caller(path: &String, props: &String) {
    let mut child = Command::new("zsh")
        .arg("-c")
        .arg(format!("qs -c {} ipc call config {}", path, props))
        .spawn()
        .expect("Failed to execute command");
    child.wait().expect("Failed to wait on child process");
}
