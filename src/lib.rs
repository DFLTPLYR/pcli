use std::process::Command;

use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub mod modules;

pub enum DesktopEnvironment {
    Niri,
    Unknown,
}

#[allow(dead_code)]
impl DesktopEnvironment {
    pub fn from_env() -> Self {
        match std::env::var("XDG_CURRENT_DESKTOP") {
            Ok(val) if val == "Niri" => DesktopEnvironment::Niri,
            _ => DesktopEnvironment::Unknown,
        }
    }
}

// Check if 'qs' process is running
fn is_qs_running() -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg("qs")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub os: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub uptime: Option<u64>,
    pub boot_time: Option<u64>,
    pub cpu: Option<SystemCPU>,
    pub memory: Option<SystemMemory>,
    pub gpu: Option<GpuInfo>,
    pub disks: Option<Vec<SystemDisk>>,
    pub network: Option<Vec<NetworkInterface>>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemCPU {
    pub cpu_architecture: String,
    pub cpu_usage: f32,
    pub cpu_frequency: u64,
    pub cpu_cores: usize,
    pub physical_cores: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SystemMemory {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub free_memory: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SystemDisk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
    pub kind: String,
    pub file_system: String,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: String,
    pub model: String,
    pub family: String,
    pub device_id: u32,
    pub total_vram: u64,
    pub used_vram: u64,
    pub free_vram: u64,
    pub temperature: f32,
    pub utilization: f32,
}

// Cli
#[derive(Subcommand)]
pub enum Commands {
    /// Get hardware information
    Hardware,
    Compositor,
    /// Shell actions
    Launch {
        #[clap(subcommand)]
        target: LaunchTarget,
    },
    GeneratePalette {
        #[clap(value_name = "PATH")]
        targets: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum LaunchTarget {
    WallpaperPicker,
    AppLauncher,
    ExtendedBar,
    ShellSettings,
}
