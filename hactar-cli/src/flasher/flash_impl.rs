use crate::config::load_stm32_configs;
use crate::flasher::{esp32_uploader::ESP32S3Uploader, stm32_uploader::STM32Uploader};
use crate::utility::colors::*;
use crate::utility::errors::{HactarError, Result};
use crate::utility::scanning::{scan_for_hactars, UartConfig};
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::fs;
use std::time::Duration;

#[derive(Debug)]
pub struct FlashArgs {
    pub port: Option<String>,
    pub baud: u32,
    pub chip: String,
    pub binary_path: Option<String>,
    pub use_external_flasher: bool,
}

enum Uploader {
    STM32(STM32Uploader),
    ESP32(ESP32S3Uploader),
}

impl Uploader {
    fn flash_select(&mut self) -> Result<()> {
        match self {
            Uploader::STM32(u) => u.flash_select(),
            Uploader::ESP32(u) => u.flash_select(),
        }
    }

    fn flash_firmware(&mut self, binary_path: &str) -> Result<bool> {
        match self {
            Uploader::STM32(u) => {
                let binary = fs::read(binary_path)?;
                let sectors = u.get_sectors_for_firmware(binary.len())?;
                u.send_extended_erase_memory(&sectors, true)?;

                let start_addr = u.chip_config.as_ref()
                    .ok_or_else(|| HactarError::Other("Chip config not set".to_string()))?
                    .usr_start_addr;

                u.send_write_memory(&binary, start_addr)?;

                // For mgmt chip, jump to the application
                if u.chip == "mgmt" {
                    u.send_go(start_addr)?;
                }

                Ok(true)
            }
            Uploader::ESP32(u) => u.flash_firmware(binary_path),
        }
    }
}

fn create_uploader(port: Box<dyn SerialPort>, chip: &str) -> Result<Uploader> {
    let chip_lower = chip.to_lowercase();

    if chip_lower == "mgmt" || chip_lower == "ui" {
        let configs = load_stm32_configs()
            .map_err(|e| HactarError::ConfigNotFound(format!("STM32 config: {}", e)))?;
        let uploader = STM32Uploader::new(port, chip_lower, configs)?;
        Ok(Uploader::STM32(uploader))
    } else if chip_lower == "net" {
        let uploader = ESP32S3Uploader::new(port, chip_lower)?;
        Ok(Uploader::ESP32(uploader))
    } else {
        Err(HactarError::UnsupportedChip(chip.to_string()))
    }
}

pub fn flash(args: FlashArgs) -> Result<()> {
    // Validate arguments
    if !args.use_external_flasher && args.binary_path.is_none() {
        return Err(HactarError::Other(
            "A binary path must be provided if not using external flasher".to_string(),
        ));
    }

    // Set up UART configuration
    let uart_config = UartConfig {
        baudrate: args.baud,
        data_bits: DataBits::Eight,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_secs(2),
    };

    // Determine which ports to use
    let ports: Vec<String> = if let Some(ref port) = args.port {
        vec![port.clone()]
    } else {
        println!("Searching for Hactar devices");
        scan_for_hactars(&uart_config)?
    };

    if ports.is_empty() {
        return Err(HactarError::NoDevicesFound);
    }

    println!("Uploading to {} Hactar device(s) on ports: {:?}", ports.len(), ports);

    // Flash each device
    for port_name in ports {
        let mut flashed = false;
        let num_attempts = 5;

        for attempt in 1..=num_attempts {
            println!("\nAttempt {}/{} for port {}", attempt, num_attempts, info(&port_name));

            match flash_device(&port_name, &uart_config, &args) {
                Ok(()) => {
                    flashed = true;
                    println!("Done Flashing {}", success("SUCCESS"));
                    break;
                }
                Err(e) => {
                    println!("{} {}, will try again", error("[Error]"), e);
                    if attempt < num_attempts {
                        std::thread::sleep(Duration::from_secs(12));
                    }
                }
            }
        }

        if !flashed {
            println!("Failed to flash {} after {} attempts", error(&port_name), num_attempts);
            return Err(HactarError::Other(format!("Failed to flash {}", port_name)));
        }
    }

    println!("\nDone Flashing {}", success("GOODBYE"));
    Ok(())
}

fn flash_device(port_name: &str, uart_config: &UartConfig, args: &FlashArgs) -> Result<()> {
    // Open serial port
    let mut port = serialport::new(port_name, uart_config.baudrate)
        .data_bits(uart_config.data_bits)
        .parity(uart_config.parity)
        .stop_bits(uart_config.stop_bits)
        .timeout(uart_config.timeout)
        .open()?;

    println!("Opened port: {} baudrate: {}", info(port_name), success(&format!("{}", uart_config.baudrate)));

    // Disable logs
    use crate::utility::commands::get_command_map;
    use std::io::{Read, Write};

    let command_map = get_command_map();
    if let Some(disable_logs) = command_map.get("disable logs") {
        port.write_all(disable_logs)?;

        // Read and discard response
        port.set_timeout(Duration::from_millis(100))?;
        let mut buf = [0u8; 1];
        while port.read(&mut buf).is_ok() {
            // Keep reading until timeout
        }
        port.set_timeout(uart_config.timeout)?;
    }

    // Create uploader
    let mut uploader = create_uploader(port, &args.chip)?;

    if args.use_external_flasher {
        uploader.flash_select()?;
    } else if let Some(binary_path) = &args.binary_path {
        uploader.flash_firmware(binary_path)?;
    }

    Ok(())
}
