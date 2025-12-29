use sysinfo::System;
use serde::Serialize;
use daemonize::Daemonize;
use std::{fs::File, thread, time::Duration};

#[derive(Serialize)]
struct MySysInfo {
    total_memory: u64,
    used_memory: u64,
}

fn main() {
    let stdout = File::create("/tmp/sysinfo.out").unwrap();
    let stderr = File::create("/tmp/sysinfo.err").unwrap();

    let daemonize = Daemonize::new()
            .pid_file("/tmp/sysinfo.pid")
            .stdout(stdout)
            .stderr(stderr);
    daemonize.start().expect("Failed to daemonize");

    let mut sys = System::new_all();

    loop {
        sys.refresh_memory();

        let info = MySysInfo {
            total_memory: sys.total_memory(),
            used_memory: sys.used_memory(),
        };

        println!("{}", serde_json::to_string(&info).unwrap());

        thread::sleep(Duration::from_secs(1));
    }

}
