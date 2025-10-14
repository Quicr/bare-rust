// ESP32-S3 Uploader Implementation
// Based on ESP32 Serial Protocol

use crate::flasher::esp32_slip_packet::ESP32SlipPacket;
use crate::flasher::uart_utils;
use colored::Colorize;
use crate::utility::errors::{HactarError, Result};
use serialport::{Parity, SerialPort};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};

// ESP32 Commands
const SYNC: u8 = 0x08;
const FLASH_BEGIN: u8 = 0x02;
const FLASH_DATA: u8 = 0x03;
const FLASH_END: u8 = 0x04;
const SPI_SET_PARAMS: u8 = 0x0B;
const SPI_ATTACH: u8 = 0x0D;
const SPI_FLASH_MD5: u8 = 0x13;

const READY: u8 = 0x80;

// Block size (1KB)
const BLOCK_SIZE: usize = 0x400;

#[derive(Debug, Deserialize, Serialize)]
pub struct FlasherArgsEntry {
    pub offset: String,
    pub file: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FlasherArgs {
    pub bootloader: Option<FlasherArgsEntry>,
    #[serde(rename = "partition-table")]
    pub partition_table: Option<FlasherArgsEntry>,
    pub app: Option<FlasherArgsEntry>,
}

pub struct ESP32S3Uploader {
    pub port: Box<dyn SerialPort>,
    pub chip: String,
}

impl ESP32S3Uploader {
    pub fn new(port: Box<dyn SerialPort>, chip: String) -> Result<Self> {
        Ok(Self { port, chip })
    }

    /// Write a packet and wait for response
    fn write_packet_wait_for_response(
        &mut self,
        packet: &mut ESP32SlipPacket,
        packet_type: u8,
        checksum: bool,
        retry_num: usize,
    ) -> Result<ESP32SlipPacket> {
        for _ in 0..retry_num {
            self.write_packet(packet, checksum)?;

            if let Ok(reply) = self.wait_for_response_packet(packet_type) {
                return Ok(reply);
            }
        }

        Err(HactarError::NoResponse)
    }

    /// Write a packet to the serial port
    fn write_packet(&mut self, packet: &mut ESP32SlipPacket, checksum: bool) -> Result<()> {
        let data = packet.to_encoded_bytes(checksum);
        self.port.write_all(&data)?;
        Ok(())
    }

    /// Wait for a response packet
    fn wait_for_response_packet(&mut self, packet_type: u8) -> Result<ESP32SlipPacket> {
        let mut in_bytes: Vec<Vec<u8>> = Vec::new();
        let mut rx_byte = vec![0u8; 1];

        loop {
            // Wait for start byte (END)
            loop {
                let bytes_read = self.port.read(&mut rx_byte)?;
                if bytes_read < 1 {
                    if !in_bytes.is_empty() {
                        return ESP32SlipPacket::from_bytes(&in_bytes);
                    }
                    return Err(HactarError::NoResponse);
                }

                if rx_byte[0] == esp32_slip_packet::END {
                    in_bytes.push(rx_byte.clone());
                    break;
                }
            }

            // Read until next END byte
            loop {
                let bytes_read = self.port.read(&mut rx_byte)?;
                if bytes_read < 1 {
                    if !in_bytes.is_empty() {
                        return ESP32SlipPacket::from_bytes(&in_bytes);
                    }
                    return Err(HactarError::NoResponse);
                }

                in_bytes.push(rx_byte.clone());

                if rx_byte[0] == esp32_slip_packet::END {
                    break;
                }
            }

            // Try to parse packet
            let packet = ESP32SlipPacket::from_bytes(&in_bytes)?;
            if packet.get(1, 1) as u8 == packet_type {
                return Ok(packet);
            } else if packet_type == 0xFF {
                // Any packet type
                return Ok(packet);
            }

            // Continue looking for the right packet type
            in_bytes.clear();
        }
    }

    /// Sync with bootloader
    pub fn sync(&mut self) -> Result<()> {
        let mut packet = ESP32SlipPacket::new(0x00, SYNC);
        packet.push_data_array(&[0x07, 0x07, 0x12, 0x20], "big");
        packet.push_data_array(&[0x55; 32], "big");

        let reply = self.write_packet_wait_for_response(&mut packet, SYNC, false, 5)?;

        if reply.get_command() == SYNC {
            println!("Activating device: {}", "SUCCESS".bright_green());
            Ok(())
        } else {
            println!("Activating device: {}", "NO REPLY".bright_yellow());
            Err(HactarError::Other("Failed to Activate device".to_string()))
        }
    }

    /// Attach SPI
    pub fn attach_spi(&mut self) -> Result<()> {
        let mut packet = ESP32SlipPacket::new(0, SPI_ATTACH);
        packet.push_data_array(&[0; 8], "big");

        let reply = self.write_packet_wait_for_response(&mut packet, SPI_ATTACH, false, 5)?;

        if reply.get_data_field().last() == Some(&1) {
            return Err(HactarError::Other("Error occurred in attach spi".to_string()));
        }

        Ok(())
    }

    /// Set SPI parameters
    pub fn set_spi_parameters(&mut self) -> Result<()> {
        let mut packet = ESP32SlipPacket::new(0, SPI_SET_PARAMS);

        // ID
        packet.push_data(0, 4);
        // Total size (4MB)
        packet.push_data(0x400000, 4);
        // ESP32-S3 block size
        packet.push_data(64 * 1024, 4);
        // ESP32-S3 sector size
        packet.push_data(4 * 1024, 4);
        // ESP32-S3 page size
        packet.push_data(256, 4);
        // Status mask
        packet.push_data(0xFFFF, 4);

        let reply = self.write_packet_wait_for_response(&mut packet, SPI_SET_PARAMS, false, 5)?;

        if reply.get_data_field().last() == Some(&1) {
            println!("Error occurred in spi set params");
        }

        Ok(())
    }

    /// Begin flash operation
    fn start_flash(&mut self, size: usize, num_blocks: usize, offset: u32) -> Result<()> {
        let mut packet = ESP32SlipPacket::new(0x00, FLASH_BEGIN);

        // Size to erase
        packet.push_data(size as u32, 4);
        // Number of incoming packets (blocks)
        packet.push_data(num_blocks as u32, 4);
        // How big each packet will be
        packet.push_data(BLOCK_SIZE as u32, 4);
        // Where to begin writing
        packet.push_data(offset, 4);
        // Just some zeroes
        packet.push_data(0, 4);

        let reply = self.write_packet_wait_for_response(&mut packet, FLASH_BEGIN, false, 5)?;

        if reply.get_data_field().last() == Some(&1) {
            return Err(HactarError::Esp32FlashError);
        }

        Ok(())
    }

    /// Write flash data
    fn write_flash(&mut self, file: &str, data: &[u8], _num_blocks: usize) -> Result<()> {
        let size = data.len();
        let mut data_ptr = 0;
        let mut packet_idx = 0;

        print!("\rFlashing: {}{:.2}%", "".bright_green(), 0.0);
        std::io::stdout().flush()?;

        while data_ptr < size {
            let mut bin_packet = ESP32SlipPacket::new(0, FLASH_DATA);

            // Push data size
            bin_packet.push_data(BLOCK_SIZE as u32, 4);
            // Push sequence number
            bin_packet.push_data(packet_idx, 4);
            // Two zeros (32-bit x 2)
            bin_packet.push_data(0, 4);
            bin_packet.push_data(0, 4);

            // Get data chunk
            let end = std::cmp::min(data_ptr + BLOCK_SIZE, size);
            let mut data_bytes = data[data_ptr..end].to_vec();

            // Pad to block size
            if data_bytes.len() < BLOCK_SIZE {
                data_bytes.resize(BLOCK_SIZE, 0xFF);
            }

            // Push data
            bin_packet.push_data_array(&data_bytes, "big");

            // Write packet
            let reply = self.write_packet_wait_for_response(&mut bin_packet, FLASH_DATA, true, 5)?;

            if reply.get_command() != FLASH_DATA {
                println!("Error occurred when writing address {} of {}", data_ptr, file);
                return Err(HactarError::Esp32FlashError);
            }

            print!("\rFlashing: {}{:.2}%", "".bright_green(), (data_ptr as f32 / size as f32) * 100.0);
            std::io::stdout().flush()?;

            data_ptr += BLOCK_SIZE;
            packet_idx += 1;
        }

        println!("\rFlashing: {}%", "100.00".bright_green());
        Ok(())
    }

    /// End flash operation
    fn end_flash(&mut self) -> Result<()> {
        let mut packet = ESP32SlipPacket::new(0, FLASH_END);
        packet.push_data(0x1, 4);

        let reply = self.write_packet_wait_for_response(&mut packet, FLASH_END, false, 5)?;

        if reply.get_command() != FLASH_END {
            println!("Failed to restart board");
        }

        println!("Flashing: {}", "COMPLETE".bright_green());
        Ok(())
    }

    /// Verify flash with MD5
    fn flash_md5(&mut self, data: &[u8], address: u32, size: usize) -> Result<()> {
        let mut packet = ESP32SlipPacket::new(0, SPI_FLASH_MD5);
        packet.push_data(address, 4);
        packet.push_data(size as u32, 4);
        packet.push_data(0, 4);
        packet.push_data(0, 4);

        let reply = self.write_packet_wait_for_response(&mut packet, SPI_FLASH_MD5, false, 5)?;

        if reply.get_data_field().last() == Some(&1) {
            return Err(HactarError::Esp32FlashError);
        }

        // Get MD5 from response
        let mut res_md5: Vec<u8> = reply.get_bytes(12, 32);
        res_md5.reverse();

        // Calculate MD5
        let result = md5::compute(data);
        let md5_hex = format!("{:x}", result);

        let loc_md5: Vec<u8> = md5_hex.bytes().collect();

        for i in 0..loc_md5.len() {
            if res_md5.get(i) != loc_md5.get(i) {
                return Err(HactarError::Md5Mismatch);
            }
        }

        Ok(())
    }

    /// Flash firmware from build directory
    pub fn flash(&mut self, build_path: &str) -> Result<()> {
        let flasher_args_path = format!("{}/flasher_args.json", build_path);
        let flasher_args: FlasherArgs = serde_json::from_str(&fs::read_to_string(&flasher_args_path)?)?;

        let mut binaries = Vec::new();

        if let Some(mut bootloader) = flasher_args.bootloader {
            bootloader.file = format!("bootloader/{}", bootloader.file);
            binaries.push(("bootloader", bootloader));
        }

        if let Some(partition_table) = flasher_args.partition_table {
            binaries.push(("partition-table", partition_table));
        }

        if let Some(app) = flasher_args.app {
            binaries.push(("app", app));
        }

        for (name, binary) in binaries {
            let file_path = format!("{}/{}", build_path, binary.file);
            let data = fs::read(&file_path)?;
            let size = data.len();
            let offset = u32::from_str_radix(binary.offset.trim_start_matches("0x"), 16)
                .map_err(|_| HactarError::Other("Invalid offset".to_string()))?;
            let num_blocks = size.div_ceil(BLOCK_SIZE);

            println!("Flashing: {}, size: {:#x}, start_addr: {:#x}", name.bright_yellow(), size, offset);

            self.start_flash(size, num_blocks, offset)?;
            self.write_flash(&binary.file, &data, num_blocks)?;
            self.flash_md5(&data, offset, size)?;
        }

        self.end_flash()?;

        Ok(())
    }

    /// Put device in flash mode
    pub fn flash_select(&mut self) -> Result<()> {
        use crate::utility::commands::get_command_map;

        let command_map = get_command_map();
        if let Some(flash_net) = command_map.get("flash net") {
            self.port.write_all(flash_net)?;
            println!("Sent command to flash Net");

            self.port.flush()?;

            uart_utils::try_pattern(&mut self.port, uart_utils::OK, 1, 5)?;
            println!("Flash Net command: {}", "CONFIRMED".bright_green());

            println!("Update uart to parity: {}", "NONE".bright_blue());
            self.port.set_parity(Parity::None)?;

            uart_utils::try_pattern(&mut self.port, READY, 1, 5)?;
            println!("Flash Net: {}", "READY".bright_blue());

            self.port.flush()?;
            self.port.clear(serialport::ClearBuffer::Input)?;

            println!("Activating NET Upload Mode: {}", "SUCCESS".bright_green());
        }

        Ok(())
    }

    /// Flash firmware (full workflow)
    pub fn flash_firmware(&mut self, binary_path: &str) -> Result<bool> {
        println!("{}", "Starting Net Upload".bright_white());

        self.flash_select()?;
        self.sync()?;
        self.attach_spi()?;
        self.set_spi_parameters()?;
        self.flash(binary_path)?;

        Ok(true)
    }
}

// Re-export for convenience
use crate::flasher::esp32_slip_packet;
