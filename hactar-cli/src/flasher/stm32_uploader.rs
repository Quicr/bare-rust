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

    /// Read memory from device
    pub fn send_read_memory(&mut self, address: u32, num_bytes: usize) -> Result<Vec<u8>> {
        self.check_init()?;

        let reply = write_byte_wait_for_ack(&mut self.port, commands::READ_MEMORY, 5, true);
        self.handle_reply(reply, "Read memory command", "Failed to read memory command", false)?;

        // Send address with checksum
        let addr_bytes = address.to_be_bytes();
        let checksum = Self::calculate_checksum(&addr_bytes);
        let mut memory_addr = addr_bytes.to_vec();
        memory_addr.push(checksum);

        let reply = write_bytes_wait_for_ack(&mut self.port, &memory_addr, 1);
        self.handle_reply(reply, "Read memory set address", "Failed to set address for read memory command", false)?;

        // Send number of bytes to read (N-1)
        let num_bytes_minus_1 = (num_bytes - 1) as u8;
        let reply = write_byte_wait_for_ack(&mut self.port, num_bytes_minus_1, 1, true);
        self.handle_reply(reply, "Read memory num bytes", "Failed to read number of bytes for memory command", false)?;

        // Read the data
        let recv_data = try_get_bytes(&mut self.port, num_bytes)?;
        Ok(recv_data)
    }

    /// Calculate sectors needed for firmware of given size
    pub fn get_sectors_for_firmware(&mut self, data_len: usize) -> Result<Vec<usize>> {
        self.check_init()?;

        let config = self.chip_config.as_ref()
            .ok_or_else(|| HactarError::Other("Chip configuration not set".to_string()))?;

        let mut remaining = data_len;
        let mut sectors = 0;
        let total_sectors = config.sectors.len();

        while remaining > 0 {
            if sectors >= total_sectors {
                return Err(HactarError::InsufficientFlash);
            }
            remaining = remaining.saturating_sub(config.sectors[sectors].size as usize);
            sectors += 1;
        }

        Ok((0..sectors).collect())
    }

    /// Extended Erase Memory command
    pub fn send_extended_erase_memory(&mut self, sectors_to_delete: &[usize], fast_verify: bool) -> Result<()> {
        self.check_init()?;

        println!("Erase: {}", info("STARTED"));
        println!("Erase  {} {}", info("SECTORS"), warning(&format!("{:?}", sectors_to_delete)));

        let mut deleted_sectors = Vec::new();

        for &sector in sectors_to_delete {
            let reply = write_byte_wait_for_ack(&mut self.port, commands::EXTENDED_ERASE, 1, true);
            self.handle_reply(reply, "\nExtended Erase", "Extended erase failed", false)?;

            // Number of sectors (0x0000 means delete 1 sector)
            let num_sectors: u16 = 0;
            let sector_num = sector as u16;

            // Build data: num_sectors (2 bytes) + sector (2 bytes) + checksum
            let mut data = Vec::new();
            data.extend_from_slice(&num_sectors.to_be_bytes());
            data.extend_from_slice(&sector_num.to_be_bytes());
            let checksum = Self::calculate_checksum(&data);
            data.push(checksum);

            print!("\rErased {} {}", info("SECTORS"), warning(&format!("{:?}", deleted_sectors)));
            std::io::Write::flush(&mut std::io::stdout())?;

            // Erasing takes time, increase timeout
            let original_timeout = self.port.timeout();
            self.port.set_timeout(Duration::from_secs(5))?;

            let reply = write_bytes_wait_for_ack(&mut self.port, &data, 1);

            // Restore timeout
            self.port.set_timeout(original_timeout)?;

            self.handle_reply(reply, "\nErase memory", "Failed to Erase", false)?;

            deleted_sectors.push(sector);
        }

        println!("\rErased {} {}", info("SECTORS"), warning(&format!("{:?}", deleted_sectors)));

        if fast_verify {
            self.fast_erase_verify(&deleted_sectors)?;
        }

        println!("Erase: {}", success("COMPLETE"));
        Ok(())
    }

    /// Fast verification of erased sectors
    fn fast_erase_verify(&mut self, sectors: &[usize]) -> Result<()> {
        self.check_init()?;

        println!("Erase Verify: {}", info("BEGIN"));

        // Clone the sector addresses to avoid borrowing issues
        let sector_addrs: Vec<u32> = {
            let config = self.chip_config.as_ref()
                .ok_or_else(|| HactarError::Other("Chip configuration not set".to_string()))?;
            sectors.iter()
                .map(|&sector| config.sectors[sector].addr)
                .collect()
        };

        let mem_bytes_sz = 256;
        let expected_mem = vec![0xFF; mem_bytes_sz];
        let num_sectors = sectors.len();

        for (i, (&sector, &addr)) in sectors.iter().zip(sector_addrs.iter()).enumerate() {
            let percent_verified = (i * 100) / num_sectors;
            print!("\rVerifying erase: {}{}% verified", success(&format!("{:2}", percent_verified)), dim(""));
            std::io::Write::flush(&mut std::io::stdout())?;

            if !self.flash_compare(&expected_mem, addr)? {
                println!("\nVerifying: {} sector [{}]", error("Failed to verify"), sector);
                return Err(HactarError::EraseVerificationFailed(sector));
            }
        }

        println!("\rVerifying erase: {}% verified", success("100"));
        println!("Erase: {}", success("COMPLETE"));
        Ok(())
    }

    /// Compare a chunk of data to flash at given address
    fn flash_compare(&mut self, chunk: &[u8], addr: u32) -> Result<bool> {
        self.check_init()?;

        const MAX_ATTEMPTS: usize = 10;
        let mut read_count = 0;

        while read_count < MAX_ATTEMPTS {
            let mem = self.send_read_memory(addr, chunk.len())?;
            if mem == chunk {
                return Ok(true);
            }
            read_count += 1;
        }

        Ok(false)
    }

    /// Write memory to device
    pub fn send_write_memory(&mut self, data: &[u8], address: u32) -> Result<()> {
        self.check_init()?;

        const MAX_NUM_BYTES: usize = 256;

        println!("Write to Memory: {}", info("STARTED"));
        println!("Address: {}", emphasis(&format!("{:#04x}", address)));
        println!("Byte Stream Size: {}", emphasis(&format!("{}", data.len())));

        let total_bytes = data.len();
        let mut data_addr = 0;
        let mut write_addr = address;

        // Set shorter timeout for writes
        let original_timeout = self.port.timeout();
        self.port.set_timeout(Duration::from_secs(1))?;

        while data_addr < total_bytes {
            let percent_flashed = (data_addr as f32 / total_bytes as f32) * 100.0;
            print!("\rFlashing: {}{:.2}%", success(""), percent_flashed);
            std::io::Write::flush(&mut std::io::stdout())?;

            let reply = write_byte_wait_for_ack(&mut self.port, commands::WRITE_MEMORY, 1, true);
            self.handle_reply(reply, "Write Command", "Failed to send Write command", false)?;

            // Send address with checksum
            let addr_bytes = write_addr.to_be_bytes();
            let checksum = Self::calculate_checksum(&addr_bytes);
            let mut write_address_bytes = addr_bytes.to_vec();
            write_address_bytes.push(checksum);

            let reply = write_bytes_wait_for_ack(&mut self.port, &write_address_bytes, 1);
            self.handle_reply(reply, "\nWrite address bytes", "Failed to write the address bytes to the chip", false)?;

            // Get the chunk
            let end_addr = std::cmp::min(data_addr + MAX_NUM_BYTES, total_bytes);
            let mut chunk = data[data_addr..end_addr].to_vec();
            let chunk_size = chunk.len();

            // Pad to multiple of 4 bytes
            while chunk.len() % 4 != 0 {
                chunk.push(0xFF);
            }

            // Prepend number of bytes (N-1)
            let num_bytes_minus_1 = (chunk.len() - 1) as u8;
            let mut write_data = vec![num_bytes_minus_1];
            write_data.extend_from_slice(&chunk);

            // Add checksum
            let checksum = Self::calculate_checksum(&write_data);
            write_data.push(checksum);

            let reply = write_bytes_wait_for_ack(&mut self.port, &write_data, 1);
            self.handle_reply(reply, &format!("\nWrite to address {:#x}", write_addr), &format!("Failed to write to address {:#x}", write_addr), false)?;

            data_addr += chunk_size;
            write_addr += chunk_size as u32;
        }

        // Restore timeout
        self.port.set_timeout(original_timeout)?;

        println!("\rFlashing: {}%", success("100.00"));

        // Verify write
        let mut verify_addr = address;
        let mut verify_data_addr = 0;

        while verify_data_addr < total_bytes {
            let percent_verified = (verify_data_addr as f32 / total_bytes as f32) * 100.0;
            print!("\rVerifying write: {}{:.2}% verified", success(""), percent_verified);
            std::io::Write::flush(&mut std::io::stdout())?;

            let end_addr = std::cmp::min(verify_data_addr + MAX_NUM_BYTES, total_bytes);
            let chunk = &data[verify_data_addr..end_addr];
            let mem = self.send_read_memory(verify_addr, chunk.len())?;

            if chunk != mem.as_slice() {
                println!("\n{} memory address {:#x}", error("Failed to verify at"), verify_addr);
                return Err(HactarError::VerificationFailed(verify_addr));
            }

            verify_addr += chunk.len() as u32;
            verify_data_addr += chunk.len();
        }

        println!("\rVerifying write: {}% verified", success("100.00"));
        println!("Write: {}", success("COMPLETE"));

        Ok(())
    }

    /// Send GO command to jump to address
    pub fn send_go(&mut self, address: u32) -> Result<()> {
        self.check_init()?;

        let reply = write_byte_wait_for_ack(&mut self.port, commands::GO, 1, true);
        self.handle_reply(reply, "Go Command", "Failed to send Go Command", false)?;

        // Send address with checksum
        let addr_bytes = address.to_be_bytes();
        let checksum = Self::calculate_checksum(&addr_bytes);
        let mut addr_with_checksum = addr_bytes.to_vec();
        addr_with_checksum.push(checksum);

        let reply = write_bytes_wait_for_ack(&mut self.port, &addr_with_checksum, 1);
        self.handle_reply(reply, &format!("Jump to address {}", info(&format!("{:#x}", address))), &format!("Failed to jump to address {:#x}", address), true)?;

        Ok(())
    }
}
