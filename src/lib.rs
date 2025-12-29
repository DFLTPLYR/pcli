use serde::{Serialize, Deserialize};

pub mod modules;

#[derive(Serialize, Deserialize)]
pub struct SystemMemory {
    pub total_memory: u64,
    pub used_memory: u64,
}

