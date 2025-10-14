// Hactar device scanning functionality

use crate::utility::colors::{error, info, success, warning};
use crate::utility::commands::get_command_map;
use crate::utility::errors::{HactarError, Result};
use serialport::{DataBits, Parity, StopBits};
use std::io::{Read, Write};
use std::time::Duration;

const HELLO_RESPONSE: &[u8] = b"HELLO, I AM A HACTAR DEVICE";

/// Configuration for UART communication
#[derive(Debug, Clone)]
pub struct UartConfig {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
    pub timeout: Duration,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_secs(2),
        }
    }
}

/// Get list of available serial ports based on platform
fn get_available_ports() -> Result<Vec<String>> {
    let ports = serialport::available_ports()?;

    let filtered_ports: Vec<String> = if cfg!(target_os = "macos") {
        // On macOS, look for /dev/cu.usbserial* ports
        ports
            .into_iter()
            .map(|p| p.port_name)
            .filter(|name| name.starts_with("/dev/cu.usbserial"))
            .collect()
    } else if cfg!(target_os = "linux") {
        // On Linux, look for /dev/ttyUSB* ports
        ports
            .into_iter()
            .map(|p| p.port_name)
            .filter(|name| name.starts_with("/dev/ttyUSB"))
            .collect()
    } else if cfg!(target_os = "windows") {
        // On Windows, include all COM ports
        ports.into_iter().map(|p| p.port_name).collect()
    } else {
        // Fallback: all ports
        ports.into_iter().map(|p| p.port_name).collect()
    };

    Ok(filtered_ports)
}

/// Scan for Hactar devices on all available serial ports
pub fn scan_for_hactars(uart_config: &UartConfig) -> Result<Vec<String>> {
    let ports = get_available_ports()?;

    println!("Ports available: {} {:?}", ports.len(), ports);

    let mut hactar_ports = Vec::new();
    let command_map = get_command_map();

    for port_name in ports {
        if let Ok(mut port) = serialport::new(&port_name, uart_config.baudrate)
            .data_bits(uart_config.data_bits)
            .parity(uart_config.parity)
            .stop_bits(uart_config.stop_bits)
            .timeout(Duration::from_millis(100))
            .open()
        {
            // Silence the chattering chips (ESP32 in particular)
            if let Some(disable_logs) = command_map.get("disable logs") {
                let _ = port.write_all(disable_logs);

                // Read and discard the response
                let mut buf = [0u8; 1];
                while port.read(&mut buf).unwrap_or(0) > 0 {
                    // Keep reading until timeout
                }
            }

            // Send "who are you" command
            if let Some(who_are_you) = command_map.get("who are you") {
                if port.write_all(who_are_you).is_ok() {
                    // Read response (3 bytes for "ok\n" + HELLO_RESPONSE)
                    let mut response = vec![0u8; 3 + HELLO_RESPONSE.len()];
                    if port.read_exact(&mut response).is_ok() {
                        // Skip the "ok\n"
                        let device_response = &response[3..];

                        if device_response == HELLO_RESPONSE {
                            println!("Device on port {} {} a Hactar!", warning(&port_name), success("is"));
                            hactar_ports.push(port_name.clone());
                        } else {
                            println!("Device on port {} {} a Hactar!", warning(&port_name), error("not"));
                        }
                    }
                }
            }

            // Restore default logging
            if let Some(default_logging) = command_map.get("default logging") {
                let _ = port.write_all(default_logging);
            }
        }
    }

    Ok(hactar_ports)
}

/// Prompt user to select a Hactar port from available devices
pub fn select_hactar_port(uart_config: &UartConfig) -> Result<String> {
    let ports = scan_for_hactars(uart_config)?;

    if ports.is_empty() {
        println!("No hactars found, exiting");
        return Err(HactarError::NoDevicesFound);
    }

    if ports.len() == 1 {
        println!("Found 1 Hactar device: {}", info(&ports[0]));
        return Ok(ports[0].clone());
    }

    // Multiple devices found - prompt user
    loop {
        println!("Hactars found: {}", ports.len());
        println!("Select a port [0-{}]", ports.len() - 1);
        for (i, port) in ports.iter().enumerate() {
            println!("{}. {}", i, port);
        }

        print!("> ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if let Ok(idx) = input.parse::<usize>() {
            if idx < ports.len() {
                return Ok(ports[idx].clone());
            }
            println!("Invalid selection, try again");
        } else {
            println!("Error: not a number entered");
        }
    }
}
