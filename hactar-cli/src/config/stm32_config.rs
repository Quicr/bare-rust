use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct FlashSector {
    pub addr: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct ChipConfig {
    pub id: u16,
    pub name: &'static str,
    pub usr_start_addr: u32,
    pub sectors: &'static [FlashSector],
}

pub type ChipConfigs = HashMap<String, ChipConfig>;

// STM32F405RGT6 - 1MB flash with variable sector sizes
const STM32F405_START_ADDR: u32 = 0x0800_0000;

const STM32F405_SECTORS: [FlashSector; 12] = [
    FlashSector { addr: STM32F405_START_ADDR,          size: 16384 },
    FlashSector { addr: STM32F405_START_ADDR + 0x4000, size: 16384 },
    FlashSector { addr: STM32F405_START_ADDR + 0x8000, size: 16384 },
    FlashSector { addr: STM32F405_START_ADDR + 0xC000, size: 16384 },
    FlashSector { addr: STM32F405_START_ADDR + 0x10000, size: 65536 },
    FlashSector { addr: STM32F405_START_ADDR + 0x20000, size: 131072 },
    FlashSector { addr: STM32F405_START_ADDR + 0x40000, size: 131072 },
    FlashSector { addr: STM32F405_START_ADDR + 0x60000, size: 131072 },
    FlashSector { addr: STM32F405_START_ADDR + 0x80000, size: 131072 },
    FlashSector { addr: STM32F405_START_ADDR + 0xA0000, size: 131072 },
    FlashSector { addr: STM32F405_START_ADDR + 0xC0000, size: 131072 },
    FlashSector { addr: STM32F405_START_ADDR + 0xE0000, size: 131072 },
];

const STM32F405RGT6: ChipConfig = ChipConfig {
    id: 1043,
    name: "STM32F405RGT6",
    usr_start_addr: STM32F405_START_ADDR,
    sectors: &STM32F405_SECTORS,
};

/// Load STM32 chip configurations
pub fn load_stm32_configs() -> Result<ChipConfigs, Box<dyn std::error::Error>> {
    let mut configs = HashMap::new();
    configs.insert(STM32F405RGT6.id.to_string(), STM32F405RGT6.clone());
    Ok(configs)
}
