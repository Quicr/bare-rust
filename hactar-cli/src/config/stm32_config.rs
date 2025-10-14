use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashSector {
    pub addr: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipConfig {
    pub id: u16,
    pub name: String,
    pub usr_start_addr: u32,
    pub sectors: Vec<FlashSector>,
}

pub type ChipConfigs = HashMap<String, ChipConfig>;

/// Load STM32 chip configurations from JSON file
pub fn load_stm32_configs() -> Result<ChipConfigs, Box<dyn std::error::Error>> {
    let config_json = include_str!("stm32_configurations.json");
    let configs: ChipConfigs = serde_json::from_str(config_json)?;
    Ok(configs)
}
