use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MySysInfo {
    pub total_memory: u64,
    pub used_memory: u64,
}

