use knuffel::Decode;
use serde::Serialize;
use serde_json;
// cargo imports
use niri_ipc::{Response, socket::Socket};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, fs, path::PathBuf, sync::Arc};

pub fn niri_ipc_listener(mut stream: UnixStream, running: Arc<AtomicBool>) {
    let mut socket = Socket::connect().expect("Error Occured");

    let reply = socket
        .send(niri_ipc::Request::EventStream)
        .expect("What the helly?!");
    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = socket.read_events();
        while running.load(Ordering::SeqCst) {
            match read_event() {
                Ok(event) => {
                    if writeln!(stream, "{}", serde_json::to_string(&event).unwrap()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
}

pub fn hyprland_ipc_listener(mut stream: UnixStream, running: Arc<AtomicBool>) {
    let socket_path = std::env::var("XDG_RUNTIME_DIR")
        .map(|dir| {
            format!(
                "{}/hypr/{}/.socket2.sock",
                dir,
                std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap_or_default()
            )
        })
        .unwrap_or_else(|_| "/tmp/hypr/.socket2.sock".to_string());
    let socket = UnixStream::connect(&socket_path).expect("Failed to connect to Hyprland socket");
    let mut reader = BufReader::new(socket);
    while running.load(Ordering::SeqCst) {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let line = line.trim();
                if let Some((event, data)) = line.split_once(">>") {
                    handle_hyprland_event(event, data, &mut stream);
                }
            }
            Err(_) => break,
        }
    }
}

fn handle_hyprland_event(event: &str, data: &str, stream: &mut UnixStream) {
    let json = match event {
        "workspace" | "workspacev2" => {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() >= 2 {
                serde_json::json!({
                    "WorkspacesChanged": {
                        "workspaces": [{"id": parts[0].parse::<i32>().unwrap_or(0), "name": parts[1]}]
                    }
                }).to_string()
            } else {
                return;
            }
        }
        "focusedmon" | "focusedmonv2" => {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() >= 2 {
                serde_json::json!({
                    "FocusedMonitor": {"name": parts[0], "workspace": parts[1]}
                })
                .to_string()
            } else {
                return;
            }
        }
        "activewindow" => {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() >= 2 {
                serde_json::json!({
                    "WindowFocusChanged": {
                        "window": {"class": parts[0], "title": parts[1]}
                    }
                })
                .to_string()
            } else {
                return;
            }
        }
        "activewindowv2" => serde_json::json!({
            "WindowFocusChanged": {"address": data}
        })
        .to_string(),
        "openwindow" => {
            let parts: Vec<&str> = data.split(',').collect();
            if parts.len() >= 4 {
                serde_json::json!({
                    "WindowOpenedOrChanged": {
                        "window": {
                            "address": parts[0],
                            "workspace": parts[1],
                            "class": parts[2],
                            "title": parts[3]
                        }
                    }
                })
                .to_string()
            } else {
                return;
            }
        }
        "closewindow" => serde_json::json!({
            "WindowClosed": {"address": data}
        })
        .to_string(),
        "createworkspace" | "destroyworkspace" | "moveworkspace" => serde_json::json!({
            "WorkspacesChanged": {"workspaces": [{"name": data}]}
        })
        .to_string(),
        "monitoradded" | "monitorremoved" => serde_json::json!({
            "MonitorChanged": {"name": data}
        })
        .to_string(),
        "fullscreen" => serde_json::json!({
            "Fullscreen": {"state": data == "1"}
        })
        .to_string(),
        "pin" => {
            let parts: Vec<&str> = data.split(',').collect();
            serde_json::json!({
                "WindowPinned": {"address": parts.get(0).unwrap_or(&""), "pinned": *parts.get(1).unwrap_or(&"false") == "true"}
            }).to_string()
        }
        "changefloatingmode" => {
            let parts: Vec<&str> = data.split(',').collect();
            serde_json::json!({
                "WindowFloating": {"address": parts.get(0).unwrap_or(&""), "floating": *parts.get(1).unwrap_or(&"false") == "true"}
            }).to_string()
        }
        _ => return,
    };
    let _ = writeln!(stream, "{}", json);
}

#[derive(Debug, Serialize, Decode)]
pub struct Config {
    #[knuffel(children(name = "window-rule"))]
    pub window_rules: Vec<WindowRule>,
}

#[derive(Debug, Serialize, Decode)]
pub struct WindowRule {
    #[knuffel(child)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_: Option<Match>,

    // Simple children with single argument
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_on_output: Option<String>,
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_maximized: Option<bool>,
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<u32>,
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry_corner_radius: Option<u32>,
    #[knuffel(child, unwrap(argument))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip_to_geometry: Option<bool>,

    #[knuffel(child)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_floating_position: Option<FloatingPosition>,
}

#[derive(Debug, Serialize, Decode)]
pub struct Match {
    #[knuffel(property)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[knuffel(property)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Decode)]
pub struct FloatingPosition {
    #[knuffel(property)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[knuffel(property)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[knuffel(property)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_to: Option<String>,
}

pub fn get_rules(mut stream: UnixStream) {
    let home = env::var("HOME").unwrap();
    let path = PathBuf::from(home).join(".config/niri/modules/rules.kdl");

    let content = fs::read_to_string(&path).unwrap();

    // Parse directly into your struct!
    let rules: Config = knuffel::parse("rules.kdl", &content).expect("Failed to parse");

    let json = serde_json::to_string_pretty(&rules.window_rules).unwrap();
    stream.write_all(json.as_bytes()).unwrap();
}
