use crate::LaunchTarget;
// use std::process::Command;

use niri_ipc::socket::Socket;

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

pub fn shellquery(target: &LaunchTarget) {
    match target {
        LaunchTarget::WallpaperPicker => wallpaper_picker(),
        LaunchTarget::AppLauncher => println!("Quickshell: Launching App Launcher"),
        LaunchTarget::ExtendedBar => println!("Quickshell: Launching Extended Bar"),
    }
}

fn wallpaper_picker() {
    match DesktopEnvironment::from_env() {
        DesktopEnvironment::Niri => {}
        DesktopEnvironment::Unknown => { /* handle others */ }
    }
}
