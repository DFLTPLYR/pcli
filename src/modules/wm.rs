use kdl::KdlDocument;
use serde::Serialize;
use serde_json;
// cargo imports
use niri_ipc::{Response, socket::Socket};
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    path::PathBuf,
};

pub fn niri_ipc_listener(mut stream: UnixStream) {
    let mut socket = Socket::connect().expect("error eeeeeh");

    let reply = socket
        .send(niri_ipc::Request::EventStream)
        .expect("What the helly?!");
    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = socket.read_events();
        while let Ok(event) = read_event() {
            writeln!(stream, "{}", serde_json::to_string(&event).unwrap()).expect("SDYBT");
        }
    }
}

#[derive(Debug, Serialize)]
struct WindowRule {
    args: Vec<String>,
    props: Vec<(String, String)>,
    children: Vec<(String, String)>,
}

pub fn get_rules(mut stream: UnixStream) {
    let home = env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))
        .unwrap();

    let path = PathBuf::from(home)
        .join(".config")
        .join("niri")
        .join("modules")
        .join("rules.kdl");

    // Re-open file to parse (since io::copy moves the cursor)
    let mut file = File::open(&path).expect("rules.kdl not found");
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    // Parse KDL
    let mut doc: KdlDocument = content.parse().expect("Failed to parse KDL");
    doc.ensure_v2();

    // Collect window rules
    let mut window_rules: Vec<WindowRule> = Vec::new();

    for node in doc.nodes() {
        if node.name().value() == "window-rule" {
            // Extract arguments (unnamed)
            let args: Vec<String> = node
                .entries()
                .iter()
                .filter_map(|e| e.value().as_string().map(|s| s.to_string()))
                .collect();

            // Extract properties (named)
            let props: Vec<(String, String)> = node
                .entries()
                .iter()
                .filter_map(|e| {
                    e.name().and_then(|n| {
                        e.value()
                            .as_string()
                            .map(|v| (n.value().to_string(), v.to_string()))
                    })
                })
                .collect();

            // Extract children nodes (name + first argument)
            let children: Vec<(String, String)> = node
                .children()
                .unwrap_or(&KdlDocument::new())
                .nodes()
                .iter()
                .filter_map(|child| {
                    let value = child.entries().get(0)?.value().as_string()?.to_string();
                    Some((child.name().value().to_string(), value))
                })
                .collect();

            window_rules.push(WindowRule {
                args,
                props,
                children,
            });
        }
    }

    // Serialize to JSON
    let json = serde_json::to_string(&window_rules).expect("Failed to serialize to JSON");

    // Send over UnixStream
    stream
        .write_all(json.as_bytes())
        .expect("Failed to write to stream");
    stream.shutdown(Shutdown::Write).unwrap();
}
