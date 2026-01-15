use crate::{DesktopEnvironment, LaunchTarget, is_qs_running};
use std::process::Command;

pub fn shell_query(target: &LaunchTarget) {
    match target {
        LaunchTarget::WallpaperPicker => compositor_action("toggleWallpaperPicker"),
        LaunchTarget::AppLauncher => compositor_action("toggleAppLauncher"),
        LaunchTarget::ExtendedBar => compositor_action("toggleExtendedBar"),
        LaunchTarget::ShellSettings => compositor_action("toggleSettingsPanel"),
    }
}

// compositor actions based on wayland compositor type
fn compositor_action(action: &str) {
    match DesktopEnvironment::from_env() {
        DesktopEnvironment::Niri => {
            let qs = is_qs_running();
            if qs {
                qs_ipc_caller(&action.to_string());
            }
        }
        DesktopEnvironment::Unknown => { /* handle others */ }
    }
}

// a wrapper to call qs ipc commands
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

// make the path relative to home directory
fn home_relative(path: &str) -> String {
    if let Ok(home) = std::env::var("HOME")
        && path.starts_with(&home)
    {
        return path.replacen(&home, "~", 1);
    }
    path.to_string()
}

// get the config path from 'qs list --all' command
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
