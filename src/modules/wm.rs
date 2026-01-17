// cargo imports
use niri_ipc::{Response, socket::Socket};
use std::{io::Write, os::unix::net::UnixStream};

pub fn niri_ipc_listener(mut stream: UnixStream) -> std::io::Result<()> {
    let mut socket = Socket::connect()?;

    let reply = socket.send(niri_ipc::Request::EventStream)?;
    if matches!(reply, Ok(Response::Handled)) {
        let mut read_event = socket.read_events();
        while let Ok(event) = read_event() {
            writeln!(stream, "{}", serde_json::to_string(&event).unwrap())?;
        }
    }

    Ok(())
}
