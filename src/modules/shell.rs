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
        LaunchTarget::ExtendedBar => extended_bar(),
    }
}

fn extended_bar() {
    match DesktopEnvironment::from_env() {
        DesktopEnvironment::Niri => {
            let mut socket = Socket::connect().expect("Failed to connect to Niri socket");
            let reply = socket.send(Request::FocusedWindow);
            let qs = is_qs_running();
            if qs {
                println!("{:?}", reply);
                qs_ipc_caller(&"toggleExtendedBar".to_string());
            }
        }
        DesktopEnvironment::Unknown => { /* handle others */ }
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
                qs_ipc_caller(&"toggleWallpaperPicker".to_string());
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

fn qs_ipc_caller(props: &String) {
    if let Some(path) = get_config_path() {
        let rel_path = home_relative(&path)
            .trim_end_matches("shell.qml")
            .to_string();
        let mut child = Command::new("zsh")
            .arg("-c")
            .arg(format!("qs -c {} ipc call config {}", rel_path, props))
            .spawn()
            .expect("Failed to execute command");
        child.wait().expect("Failed to wait on child process");
    } else {
        eprintln!("Config path not found.");
    }
}
fn home_relative(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME")
        && path.starts_with(&home)
    {
        return path.replacen(&home, "~", 1);
    }
    path.to_string()
}

fn get_config_path() -> Option<String> {
    let output = Command::new("qs").arg("list").arg("--all").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("  Config path: ") {
            return Some(path.trim().to_string());
        }
    }
    None
}
