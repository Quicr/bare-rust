// STM32 Bootloader Protocol Implementation
// Based on AN3155: USART protocol used in the STM32 bootloader

use crate::config::{ChipConfig, ChipConfigs};
use crate::flasher::uart_utils::*;
use crate::utility::colors::*;
use crate::utility::errors::{HactarError, Result};
use serialport::{Parity, SerialPort};
use std::time::Duration;

const ACK: u8 = 0x79;
const NACK: u8 = 0x1F;

// STM32 Bootloader Commands
mod commands {
    pub const SYNC: u8 = 0x7F;
    pub const GET: u8 = 0x00;
    pub const GET_VERSION: u8 = 0x01;
    pub const GET_ID: u8 = 0x02;
    pub const READ_MEMORY: u8 = 0x11;
    pub const GO: u8 = 0x21;
    pub const WRITE_MEMORY: u8 = 0x31;
    pub const ERASE: u8 = 0x43;
    pub const EXTENDED_ERASE: u8 = 0x44;
    pub const WRITE_PROTECT: u8 = 0x63;
    pub const WRITE_UNPROTECT: u8 = 0x73;
    pub const READOUT_PROTECT: u8 = 0x82;
    pub const READOUT_UNPROTECT: u8 = 0x92;
}

pub struct STM32Uploader {
    pub port: Box<dyn SerialPort>,
    pub chip: String,
    synced: bool,
    chip_id: Option<u16>,
    pub chip_config: Option<ChipConfig>,
    configs: ChipConfigs,
}

impl STM32Uploader {
    pub fn new(port: Box<dyn SerialPort>, chip: String, configs: ChipConfigs) -> Result<Self> {
        Ok(Self {
            port,
            chip,
            synced: false,
            chip_id: None,
            chip_config: None,
            configs,
        })
    }

    /// Calculate XOR checksum for a byte array
    pub fn calculate_checksum(data: &[u8]) -> u8 {
        data.iter().fold(0u8, |acc, &b| acc ^ b)
    }

    /// Handle bootloader reply (ACK/NACK/NO_REPLY)
    fn handle_reply(&mut self, reply: Result<u8>, caller: &str, exception_str: &str, output_success: bool) -> Result<bool> {
        match reply {
            Ok(ACK) => {
                if output_success {
                    println!("{}: {}", caller, success("SUCCESSFUL"));
                }
                Ok(true)
            }
            Ok(NACK) => {
                println!("{}: {}", caller, error("FAILED"));
                self.synced = false;
                Err(HactarError::Nack)
            }
            Err(_) => {
                println!("{}: {}", caller, warning("NO REPLY"));
                self.synced = false;
                Err(HactarError::NoResponse)
            }
            Ok(_) => {
                Err(HactarError::Other(format!("{} - unexpected response", exception_str)))
            }
        }
    }

    /// Send sync byte and wait for ACK
    pub fn send_sync(&mut self, retry_num: usize) -> Result<()> {
        for attempt in 0..retry_num {
            self.port.write_all(&[commands::SYNC])?;

            match get_bytes(&mut self.port, 1) {
                Ok(ACK) => {
                    self.synced = true;
                    println!("Sync: {}", success("SUCCESSFUL"));
                    return Ok(());
                }
                Ok(resp) => {
                    if attempt == retry_num - 1 {
                        println!("Sync got unexpected response: {:#x}", resp);
                    }
                }
                Err(_) => {
                    if attempt == retry_num - 1 {
                        println!("Sync: {}", error("NO RESPONSE"));
                    }
                }
            }
        }

        self.synced = false;
        Err(HactarError::SyncFailed)
    }

    /// Check if we're synced, if not, try to sync
    pub fn check_sync(&mut self) -> Result<()> {
        if self.synced {
            return Ok(());
        }
        self.send_sync(5)
    }

    /// Check if we have chip ID, if not, get it
    pub fn check_chip_id(&mut self) -> Result<()> {
        if self.chip_id.is_some() {
            return Ok(());
        }
        self.send_get_id()
    }

    /// Check both sync and chip ID
    pub fn check_init(&mut self) -> Result<()> {
        self.check_sync()?;
        self.check_chip_id()?;
        Ok(())
    }

    /// Get chip ID
    pub fn send_get_id(&mut self) -> Result<()> {
        self.check_sync()?;

        let reply = write_byte_wait_for_ack(&mut self.port, commands::GET_ID, 5, true);
        self.handle_reply(reply, "Get ID command", "ACK was not received", false)?;

        // Get the number of incoming bytes
        let num_bytes = try_get_bytes(&mut self.port, 1)?[0] as usize;

        // Get the PID which should be N+1 bytes
        let pid_bytes = try_get_bytes(&mut self.port, num_bytes + 1)?;

        // Wait for an ACK
        let reply = get_bytes(&mut self.port, 1);
        self.handle_reply(reply, "GetID PID ACK", "Failed to get PID", false)?;

        // Convert PID bytes to u16 (big-endian)
        let pid = u16::from_be_bytes([pid_bytes[0], pid_bytes[1]]);
        self.chip_id = Some(pid);

        println!("Chip ID: {}", highlight(&format!("{:#x}", pid)));

        self.set_chip_config(pid)?;

        Ok(())
    }

    /// Set chip configuration based on chip ID
    fn set_chip_config(&mut self, pid: u16) -> Result<()> {
        println!("Retrieving configurations for chip ID: {}", highlight(&format!("{:#x}", pid)));

        let pid_str = pid.to_string();
        if let Some(config) = self.configs.get(&pid_str) {
            self.chip_config = Some(config.clone());
            println!("Found configuration for: {}", info(&config.name));
            Ok(())
        } else {
            Err(HactarError::UnknownChipId(pid))
        }
    }

    /// Get available commands from bootloader
    pub fn send_get(&mut self) -> Result<Vec<String>> {
        self.check_init()?;

        let reply = write_byte_wait_for_ack(&mut self.port, commands::GET, 5, true);
        self.handle_reply(reply, "Get Commands", "Failed to retrieve commands available to this chip", false)?;

        // Read the number of bytes - 1
        let num_bytes = try_get_bytes(&mut self.port, 1)?[0] as usize;

        // Bootloader version
        let bootloader_version = try_get_bytes(&mut self.port, 1)?[0];

        // Get all available commands
        let recv_commands = try_get_bytes(&mut self.port, num_bytes)?;

        // Wait for an ACK
        let reply = get_bytes(&mut self.port, 1);
        self.handle_reply(reply, "Get Commands Receive", "Failed to get available commands", false)?;

        println!("Bootloader version: {}", emphasis(&format!("{}", bootloader_version)));

        // Map received command bytes to names
        let command_names = self.map_commands_to_names(&recv_commands);

        print!("{} available commands: ", num_bytes);
        for (i, cmd) in command_names.iter().enumerate() {
            if i == command_names.len() - 1 {
                println!("{}", cmd);
            } else {
                print!("{}, ", cmd);
            }
        }

        Ok(command_names)
    }

    /// Map command bytes to their string names
    fn map_commands_to_names(&self, commands: &[u8]) -> Vec<String> {
        let mut names = Vec::new();
        let cmd_map = [
            (commands::SYNC, "sync"),
            (commands::GET, "get"),
            (commands::GET_VERSION, "get_version_and_read_protection_status"),
            (commands::GET_ID, "get_id"),
            (commands::READ_MEMORY, "read_memory"),
            (commands::GO, "go"),
            (commands::WRITE_MEMORY, "write_memory"),
            (commands::ERASE, "erase"),
            (commands::EXTENDED_ERASE, "extended_erase"),
            (commands::WRITE_PROTECT, "write_protect"),
            (commands::WRITE_UNPROTECT, "write_unprotect"),
            (commands::READOUT_PROTECT, "readout_protect"),
            (commands::READOUT_UNPROTECT, "readout_unprotect"),
        ];

        for &cmd_byte in commands {
            for &(byte, name) in &cmd_map {
                if cmd_byte == byte {
                    names.push(name.to_string());
                    break;
                }
            }
        }

        names
    }

    /// Put device in bootloader mode (chip-specific)
    pub fn flash_select(&mut self) -> Result<()> {
        use crate::utility::commands::get_command_map;

        if self.chip == "mgmt" {
            self.port.set_parity(Parity::Even)?;
            println!("Updated uart to parity: {}", info("EVEN"));
            println!("User, put Hactar into bootloader mode!!");
            println!("Press enter once it is done...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            self.port.clear(serialport::ClearBuffer::Input)?;
        } else if self.chip == "ui" {
            let command_map = get_command_map();
            if let Some(flash_ui) = command_map.get("flash ui") {
                self.port.write_all(flash_ui)?;
                println!("Sent command to flash UI");

                self.port.flush()?;

                try_pattern(&mut self.port, OK, 1, 5)?;
                println!("Flash UI command: {}", success("CONFIRMED"));

                println!("Update uart to parity: {}", info("EVEN"));
                self.port.set_parity(Parity::Even)?;

                try_pattern(&mut self.port, READY, 1, 5)?;
                println!("Flash UI: {}", info("READY"));

                self.port.flush()?;
                self.port.clear(serialport::ClearBuffer::Input)?;

                println!("Activating UI Upload Mode: {}", success("SUCCESS"));

                std::thread::sleep(Duration::from_secs(1));
            }
        }

        Ok(())
    }
}
