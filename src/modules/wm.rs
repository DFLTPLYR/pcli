use knuffel::Decode;
use serde::Serialize;
use serde_json;
// cargo imports
use niri_ipc::{socket::Socket, Response};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, fs, io::Write, os::unix::net::UnixStream, path::PathBuf, sync::Arc};

pub fn niri_ipc_listener(mut stream: UnixStream, running: Arc<AtomicBool>) {
    let mut socket = Socket::connect().expect("error eeeeeh");

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
