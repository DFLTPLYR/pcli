// cargo imports
use gfxinfo::active_gpu;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
    time::Duration,
};
use sysinfo::{Disks, Networks, System};

// local imports
use pcli::{DesktopEnvironment, GpuInfo, SystemCPU, SystemMemory, SystemStatus};

fn main() {
    let socket_path = "/tmp/sysinfo.sock";
    let _ = fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path).unwrap();
    for stream in listener.incoming().flatten() {
        thread::spawn(move || {
            handle_client(stream);
        });
    }
}

fn handle_client(stream: UnixStream) {
    let mut reader = BufReader::new(&stream);
    let mut request = String::new();
    if reader.read_line(&mut request).is_ok() {
        match request.trim() {
            "hardware_info" => {
                let _ = get_hardware_info(stream);
            }
            "compositor_data" => {
                match DesktopEnvironment::from_env() {
                    DesktopEnvironment::Niri => {}
                    DesktopEnvironment::Unknown => { /* handle others */ }
                }
            }
            other => {
                println!("Unknown request: {}", other);
            }
        }
    }
}

fn get_hardware_info(mut stream: UnixStream) -> std::io::Result<()> {
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();

    loop {
        sys.refresh_all();
        disks.refresh(true);
        networks.refresh(true);

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

        let network = networks
            .iter()
            .map(|(name, data)| pcli::NetworkInterface {
                name: name.to_string(),
                received_bytes: data.received(),
                transmitted_bytes: data.transmitted(),
            })
            .collect::<Vec<_>>();

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
            os: Some(System::name().unwrap_or_else(|| "<unknown>".to_owned())),
            kernel_version: Some(
                System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
            ),
            os_version: Some(System::os_version().unwrap_or_else(|| "<unknown>".to_owned())),
            uptime: Some(System::uptime()),
            boot_time: Some(System::boot_time()),
            cpu: Some(cpu),
            memory: Some(memory),
            gpu: Some(gpu),
            disks: Some(disks),
            network: Some(network),
        };

        let json = serde_json::to_string(&system_stats).unwrap();
        let _ = writeln!(stream, "{}", json);
        thread::sleep(Duration::from_secs(1));
    }
}
