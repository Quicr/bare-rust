use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FlashSector {
    pub addr: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct ChipConfig {
    pub id: u16,
    pub name: String,
    pub usr_start_addr: u32,
    pub sectors: Vec<FlashSector>,
}

pub type ChipConfigs = HashMap<String, ChipConfig>;

/// STM32F072C8T6 - 64KB flash with uniform 2KB sectors
fn stm32f072c8t6_config() -> ChipConfig {
    let start_addr = 0x0800_0000;
    let sector_size = 2048;
    let num_sectors = 64;

    let mut sectors = Vec::with_capacity(num_sectors);
    for i in 0..num_sectors {
        sectors.push(FlashSector {
            addr: start_addr + (i as u32 * sector_size),
            size: sector_size,
        });
    }

    ChipConfig {
        id: 1096,
        name: "STM32F072C8T6".to_string(),
        usr_start_addr: start_addr,
        sectors,
    }
}

/// STM32F405RGT6 - 1MB flash with variable sector sizes
fn stm32f405rgt6_config() -> ChipConfig {
    let start_addr = 0x0800_0000;

    let sectors = vec![
        FlashSector { addr: start_addr, size: 16384 },
        FlashSector { addr: start_addr + 0x4000, size: 16384 },
        FlashSector { addr: start_addr + 0x8000, size: 16384 },
        FlashSector { addr: start_addr + 0xC000, size: 16384 },
        FlashSector { addr: start_addr + 0x10000, size: 65536 },
        FlashSector { addr: start_addr + 0x20000, size: 131072 },
        FlashSector { addr: start_addr + 0x40000, size: 131072 },
        FlashSector { addr: start_addr + 0x60000, size: 131072 },
        FlashSector { addr: start_addr + 0x80000, size: 131072 },
        FlashSector { addr: start_addr + 0xA0000, size: 131072 },
        FlashSector { addr: start_addr + 0xC0000, size: 131072 },
        FlashSector { addr: start_addr + 0xE0000, size: 131072 },
    ];

    ChipConfig {
        id: 1043,
        name: "STM32F405RGT6".to_string(),
        usr_start_addr: start_addr,
        sectors,
    }
}

/// STM32F411 - 512KB flash with variable sector sizes
fn stm32f411_config() -> ChipConfig {
    let start_addr = 0x0800_0000;

    let sectors = vec![
        FlashSector { addr: start_addr, size: 16384 },
        FlashSector { addr: start_addr + 0x4000, size: 16384 },
        FlashSector { addr: start_addr + 0x8000, size: 16384 },
        FlashSector { addr: start_addr + 0xC000, size: 16384 },
        FlashSector { addr: start_addr + 0x10000, size: 65536 },
        FlashSector { addr: start_addr + 0x20000, size: 131072 },
        FlashSector { addr: start_addr + 0x40000, size: 131072 },
        FlashSector { addr: start_addr + 0x60000, size: 131072 },
    ];

    ChipConfig {
        id: 1073,
        name: "STM32F411".to_string(),
        usr_start_addr: start_addr,
        sectors,
    }
}

/// Load STM32 chip configurations
pub fn load_stm32_configs() -> Result<ChipConfigs, Box<dyn std::error::Error>> {
    let mut configs = HashMap::new();

    let f072 = stm32f072c8t6_config();
    configs.insert(f072.id.to_string(), f072);

    let f405 = stm32f405rgt6_config();
    configs.insert(f405.id.to_string(), f405);

    let f411 = stm32f411_config();
    configs.insert(f411.id.to_string(), f411);

    Ok(configs)
}
