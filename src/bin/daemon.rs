use std::{
    fs,
    io::Write,
    os::unix::net::UnixListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use sysinfo::System;
use pcli::SystemMemory;

fn main() {
    let socket_path = "/tmp/sysinfo.sock";
    let _ = fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path).unwrap();
    let clients = Arc::new(Mutex::new(Vec::new()));

    let clients_accept = Arc::clone(&clients);
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            clients_accept.lock().unwrap().push(stream);
        }
    });

    let mut sys = System::new_all();

    loop {
        let mut locked_clients = clients.lock().unwrap();
        if !locked_clients.is_empty() {
            sys.refresh_memory();

            let info = SystemMemory {
                total_memory: sys.total_memory(),
                used_memory: sys.used_memory(),
            };

            let json = serde_json::to_string(&info).unwrap();

            locked_clients.retain(|mut client| {
                writeln!(client, "{}", json).is_ok()
            });
        }
        drop(locked_clients);
        thread::sleep(Duration::from_secs(1));
    }
}

