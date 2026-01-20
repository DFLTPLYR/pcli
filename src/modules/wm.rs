use serde_json;
// cargo imports
use niri_ipc::{Response, socket::Socket};
use std::{
    fs::File,
    io::{self, Write},
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

pub fn get_rules(mut stream: UnixStream) {
    let home = std::env::var("HOME")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))
        .unwrap();

    let path = PathBuf::from(home)
        .join(".config")
        .join("niri")
        .join("modules")
        .join("rules.kdl");

    let mut file = File::open(&path).expect("no file gang");
    io::copy(&mut file, &mut stream).unwrap();

    stream.shutdown(std::net::Shutdown::Write).unwrap();
}
