use crate::config::load_stm32_configs;
use crate::flasher::{esp32_uploader::ESP32S3Uploader, stm32_uploader::STM32Uploader};
use crate::utility::errors::{HactarError, Result};
use crate::utility::scanning::{scan_for_hactars, UartConfig};
use colored::Colorize;
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
        flash_device(&port_name, &uart_config, &args)?;
        println!("Done Flashing {}", "SUCCESS".bright_green());
    }

    println!("\nDone Flashing {}", "GOODBYE".bright_green());
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

    println!("Opened port: {} baudrate: {}", port_name.bright_blue(), uart_config.baudrate.to_string().bright_green());

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

    // Flash based on chip type
    let chip_lower = args.chip.to_lowercase();

    if chip_lower == "mgmt" || chip_lower == "ui" {
        flash_stm32(port, &chip_lower, args)?;
    } else if chip_lower == "net" {
        flash_esp32(port, &chip_lower, args)?;
    } else {
        return Err(HactarError::UnsupportedChip(args.chip.clone()));
    }

    Ok(())
}

fn flash_stm32(port: Box<dyn SerialPort>, chip: &str, args: &FlashArgs) -> Result<()> {
    let configs = load_stm32_configs()
        .map_err(|e| HactarError::ConfigNotFound(format!("STM32 config: {}", e)))?;
    let mut uploader = STM32Uploader::new(port, chip.to_string(), configs)?;

    if args.use_external_flasher {
        uploader.flash_select()?;
    } else if let Some(binary_path) = &args.binary_path {
        let binary = fs::read(binary_path)?;
        let sectors = uploader.get_sectors_for_firmware(binary.len())?;
        uploader.send_extended_erase_memory(&sectors, true)?;

        let start_addr = uploader.chip_config.as_ref()
            .ok_or_else(|| HactarError::Other("Chip config not set".to_string()))?
            .usr_start_addr;

        uploader.send_write_memory(&binary, start_addr)?;

        // For mgmt chip, jump to the application
        if chip == "mgmt" {
            uploader.send_go(start_addr)?;
        }
    }

    Ok(())
}

fn flash_esp32(port: Box<dyn SerialPort>, chip: &str, args: &FlashArgs) -> Result<()> {
    let mut uploader = ESP32S3Uploader::new(port, chip.to_string())?;

    if args.use_external_flasher {
        uploader.flash_select()?;
    } else if let Some(binary_path) = &args.binary_path {
        uploader.flash_firmware(binary_path)?;
    }

    Ok(())
}
