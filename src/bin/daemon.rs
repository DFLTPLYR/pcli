// cargo imports
use gfxinfo::active_gpu;
use std::{
    fs,
    io::Write,
    os::unix::net::UnixListener,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use sysinfo::{Disks, System};

// local imports
use pcli::{GpuInfo, SystemCPU, SystemMemory, SystemStatus};

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
                cpu_usage: sys.global_cpu_usage(),
                cpu_frequency: sys.cpus().get(0).map(|c| c.frequency()).unwrap_or(0),
                physical_cores: sysinfo::System::physical_core_count().unwrap_or(0),
                cpu_cores: sys.cpus().len(),
            };

            let memory = SystemMemory {
                total_memory: sys.total_memory(),
                used_memory: sys.used_memory(),
                free_memory: sys.free_memory(),
                total_swap: sys.total_swap(),
                used_swap: sys.used_swap(),
            };

            let gpudata = active_gpu().expect("Failed to get active GPU");
            let gpuinfo = gpudata.info();

            let gpu = GpuInfo {
                vendor: gpudata.vendor().to_string(),
                model: gpudata.model().to_string(),
                family: gpudata.family().to_string(),
                device_id: *gpudata.device_id(),
                total_vram: gpuinfo.total_vram(),
                used_vram: gpuinfo.used_vram(),
                free_vram: gpuinfo.total_vram() - gpuinfo.used_vram(),
                temperature: gpuinfo.temperature() as f32 / 1000.0,
                utilization: gpuinfo.load_pct() as f32,
            };

            let disks = Disks::new_with_refreshed_list();
            let disks = disks
                .list()
                .iter()
                .map(|disk| pcli::SystemDisk {
                    name: disk.name().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    kind: format!("{:?}", disk.kind()),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                })
                .collect::<Vec<_>>();

            let system_stats = SystemStatus {
                name: System::name().unwrap_or_else(|| "<unknown>".to_owned()),
                kernel_version: System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
                os_version: System::os_version().unwrap_or_else(|| "<unknown>".to_owned()),
                uptime: System::uptime(),
                boot_time: System::boot_time(),
                cpu: cpu,
                memory: memory,
                gpu: gpu,
                disks: disks,
            };

            let json = serde_json::to_string(&system_stats).unwrap();

            locked_clients.retain(|mut client| writeln!(client, "{}", json).is_ok());
        }
        drop(locked_clients);
        thread::sleep(Duration::from_secs(1));
    }
}
