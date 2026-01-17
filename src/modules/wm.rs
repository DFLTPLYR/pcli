// cargo imports
use niri_ipc::{Response, socket::Socket};
use std::{io::Write, os::unix::net::UnixStream};

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
