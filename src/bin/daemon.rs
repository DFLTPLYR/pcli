use std::{
    fs,
    io::Write,
    os::unix::net::UnixListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use sysinfo::System;

use pcli::{SystemCPU, SystemMemory, SystemStatus};

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
            sys.refresh_all();

            let cpu = SystemCPU {
                cpu_architecture: std::env::consts::ARCH.to_string(),
                physical_cores: sysinfo::System::physical_core_count().unwrap_or(0),
                cpu_usage: sys.global_cpu_usage(),
                cpu_frequency: sys.cpus().get(0).map(|c| c.frequency()).unwrap_or(0),
                cpu_cores: sys.cpus().len(),
            };

            let memory = SystemMemory {
                total_memory: sys.total_memory(),
                used_memory: sys.used_memory(),
                total_swap: sys.total_swap(),
                used_swap: sys.used_swap(),
                free_memory: sys.free_memory(),
            };

            let system_stats = SystemStatus {
                name: System::name().unwrap_or_else(|| "<unknown>".to_owned()),
                kernel_version: System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
                os_version: System::os_version().unwrap_or_else(|| "<unknown>".to_owned()),
                uptime: System::uptime(),
                boot_time: System::boot_time(),
                cpu: cpu,
                memory: memory,
            };

            let json = serde_json::to_string(&system_stats).unwrap();

            locked_clients.retain(|mut client| writeln!(client, "{}", json).is_ok());
        }
        drop(locked_clients);
        thread::sleep(Duration::from_secs(1));
    }
}
