use serde::{Serialize, Deserialize};

pub mod modules;

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub uptime: u64,
    pub boot_time: u64,
    pub cpu: SystemCPU,
    pub memory: SystemMemory,
}

#[derive(Serialize, Deserialize)]
pub struct SystemCPU {
    pub cpu_architecture: String,
    pub physical_cores: usize,
    pub cpu_usage: f32,
    pub cpu_frequency: u64,
    pub cpu_cores: usize,
}

#[derive(Serialize, Deserialize)]
pub struct SystemMemory {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub free_memory: u64,
}

