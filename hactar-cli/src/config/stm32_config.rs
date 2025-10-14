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

// STM32F072C8T6 - 64KB flash with uniform 2KB sectors
const STM32F072_START_ADDR: u32 = 0x0800_0000;
const STM32F072_SECTOR_SIZE: u32 = 2048;

const STM32F072_SECTORS: [FlashSector; 64] = {
    let mut sectors = [FlashSector { addr: 0, size: 0 }; 64];
    let mut i = 0;
    while i < 64 {
        sectors[i] = FlashSector {
            addr: STM32F072_START_ADDR + (i as u32 * STM32F072_SECTOR_SIZE),
            size: STM32F072_SECTOR_SIZE,
        };
        i += 1;
    }
    sectors
};

const STM32F072C8T6: ChipConfig = ChipConfig {
    id: 1096,
    name: "STM32F072C8T6",
    usr_start_addr: STM32F072_START_ADDR,
    sectors: &STM32F072_SECTORS,
};

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

    configs.insert(STM32F072C8T6.id.to_string(), STM32F072C8T6.clone());
    configs.insert(STM32F405RGT6.id.to_string(), STM32F405RGT6.clone());

    Ok(configs)
}
