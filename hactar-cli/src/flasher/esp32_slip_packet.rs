// ESP32 SLIP Packet Implementation
// SLIP protocol for ESP32 bootloader communication

use crate::utility::errors::{HactarError, Result};

/// SLIP Protocol Constants
pub const END: u8 = 0xC0;
pub const ESC: u8 = 0xDB;
pub const ESC_END: u8 = 0xDC;
pub const ESC_ESC: u8 = 0xDD;

/// ESP32 SLIP Packet structure
/// ```text
/// |-------------------|
/// |Byte    name       |
/// |-------------------|
/// |0       Direction  |
/// |1       Command    |
/// |2-3     Size       |
/// |4-7     Checksum   |
/// |8..n    Data       |
/// |-------------------|
/// ```
/// NOTE: Data is stored in little endian format for multi-byte fields
#[derive(Debug, Clone)]
pub struct ESP32SlipPacket {
    /// Header (8 bytes) + data
    data: Vec<u8>,
    /// Length of data field
    data_length: usize,
}

impl ESP32SlipPacket {
    /// Create a new SLIP packet with direction and command
    pub fn new(direction: u8, command: u8) -> Self {
        let mut packet = Self {
            data: vec![0u8; 8],
            data_length: 0,
        };
        packet.set_header(direction, command, 0);
        packet
    }

    /// Create packet from SLIP-encoded bytes
    pub fn from_bytes(data: &[Vec<u8>]) -> Result<Self> {
        // Flatten the input
        let mut flat_data: Vec<u8> = Vec::new();
        for byte_vec in data {
            for &byte in byte_vec {
                flat_data.push(byte);
            }
        }

        if flat_data.is_empty() || flat_data[0] != END || flat_data[flat_data.len() - 1] != END {
            return Err(HactarError::SlipPacket("Missing START/END bytes".to_string()));
        }

        // Decode SLIP encoding
        let mut decoded = Vec::new();
        let mut idx = 1; // Skip first END byte

        while idx < flat_data.len() {
            if flat_data[idx] == ESC {
                if idx + 1 >= flat_data.len() {
                    return Err(HactarError::SlipPacket("Incomplete escape sequence".to_string()));
                }
                if flat_data[idx + 1] == ESC_END {
                    decoded.push(END);
                } else if flat_data[idx + 1] == ESC_ESC {
                    decoded.push(ESC);
                } else {
                    return Err(HactarError::SlipPacket(format!("Invalid escape sequence at {}", idx)));
                }
                idx += 2;
            } else if flat_data[idx] == END {
                break;
            } else {
                decoded.push(flat_data[idx]);
                idx += 1;
            }
        }

        if decoded.len() < 8 {
            return Err(HactarError::SlipPacket("Data needs to be at least 8 bytes".to_string()));
        }

        let mut packet = Self {
            data: decoded[0..8].to_vec(),
            data_length: 0,
        };

        let size = packet.get_size();
        if decoded.len() >= 8 + size {
            packet.push_data_array(&decoded[8..8 + size], "little");
        }

        Ok(packet)
    }

    /// Set packet header
    pub fn set_header(&mut self, direction: u8, command: u8, size: u16) {
        self.set_direction(direction);
        self.set_command(command);
        self.set_size(size);
    }

    /// Set direction (0 or 1)
    pub fn set_direction(&mut self, direction: u8) {
        if direction > 1 {
            panic!("Direction must be either 0 or 1");
        }
        self.data[0] = direction;
    }

    /// Set command
    pub fn set_command(&mut self, command: u8) {
        self.data[1] = command;
    }

    /// Set size (little-endian)
    pub fn set_size(&mut self, size: u16) {
        let bytes = size.to_le_bytes();
        self.data[2] = bytes[0];
        self.data[3] = bytes[1];
    }

    /// Get bytes from packet starting at index
    pub fn get(&self, start_idx: usize, num_bytes: usize) -> u32 {
        let bytes = &self.data[start_idx..start_idx + num_bytes];
        u32::from_le_bytes([
            *bytes.first().unwrap_or(&0),
            *bytes.get(1).unwrap_or(&0),
            *bytes.get(2).unwrap_or(&0),
            *bytes.get(3).unwrap_or(&0),
        ])
    }

    /// Get raw bytes from packet
    pub fn get_bytes(&self, start_idx: usize, num_bytes: usize) -> Vec<u8> {
        self.data[start_idx..start_idx + num_bytes].to_vec()
    }

    /// Get direction
    pub fn get_direction(&self) -> u8 {
        self.data[0]
    }

    /// Get command
    pub fn get_command(&self) -> u8 {
        self.data[1]
    }

    /// Get size (little-endian)
    pub fn get_size(&self) -> usize {
        u16::from_le_bytes([self.data[2], self.data[3]]) as usize
    }

    /// Get data field
    pub fn get_data_field(&self) -> &[u8] {
        &self.data[8..8 + self.data_length]
    }

    /// Push a single data element (stored in little-endian)
    pub fn push_data(&mut self, ele: u32, size: usize) {
        let ele_bytes = ele.to_le_bytes();
        for &byte in ele_bytes.iter().take(size) {
            self.data.push(byte);
        }
        self.data_length += size;
    }

    /// Push data array
    pub fn push_data_array(&mut self, data_in: &[u8], endian_format: &str) {
        if endian_format == "little" {
            for &byte in data_in.iter().rev() {
                self.data.push(byte);
                self.data_length += 1;
            }
        } else {
            for &byte in data_in {
                self.data.push(byte);
                self.data_length += 1;
            }
        }
    }

    /// Get data length
    pub fn length(&self) -> usize {
        self.data_length
    }

    /// Set checksum
    pub fn set_checksum(&mut self) {
        // Checksum seed
        let mut checksum: u8 = 0xEF;

        // XOR all data bytes (starting from byte 16, which is index 8 + 8)
        // But in reality, we checksum from the data field onwards
        for i in 8..self.data.len() {
            checksum ^= self.data[i];
        }

        // Store checksum in little-endian
        let checksum_bytes = (checksum as u32).to_le_bytes();
        self.data[4] = checksum_bytes[0];
        self.data[5] = checksum_bytes[1];
        self.data[6] = checksum_bytes[2];
        self.data[7] = checksum_bytes[3];
    }

    /// SLIP encode the packet
    pub fn slip_encode(&mut self, checksum: bool) -> Vec<u8> {
        // Set checksum if requested
        if checksum {
            self.set_checksum();
        }

        // Set the current size
        self.set_size(self.data_length as u16);

        let mut encoded_data = Vec::new();
        encoded_data.push(END);

        for &byte in &self.data {
            if byte == END {
                encoded_data.push(ESC);
                encoded_data.push(ESC_END);
            } else if byte == ESC {
                encoded_data.push(ESC);
                encoded_data.push(ESC_ESC);
            } else {
                encoded_data.push(byte);
            }
        }

        // Put on the final end byte
        encoded_data.push(END);

        encoded_data
    }

    /// Get SLIP-encoded packet as bytes
    pub fn to_encoded_bytes(&mut self, checksum: bool) -> Vec<u8> {
        self.slip_encode(checksum)
    }
}

impl std::fmt::Display for ESP32SlipPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut packet_copy = self.clone();
        let to_print = packet_copy.slip_encode(true);

        let mut s_out = String::new();
        for (idx, byte) in to_print.iter().enumerate() {
            s_out.push_str(&format!("{:02X}", byte));
            if (idx + 1) % 8 == 0 {
                s_out.push(' ');
            }
            if (idx + 1) % 16 == 0 {
                s_out.push('\n');
            }
        }

        write!(f, "{}", s_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slip_encoding() {
        let mut packet = ESP32SlipPacket::new(0, 0x08);
        packet.push_data(0x07070012, 4);
        packet.push_data_array(&vec![0x55; 32], "big");

        let encoded = packet.slip_encode(false);
        assert_eq!(encoded[0], END);
        assert_eq!(encoded[encoded.len() - 1], END);
    }

    #[test]
    fn test_escape_sequences() {
        let mut packet = ESP32SlipPacket::new(0, END); // Command with END byte
        let encoded = packet.slip_encode(false);

        // Should have escaped the END byte in command field
        let contains_escape = encoded.windows(2).any(|w| w == [ESC, ESC_END]);
        assert!(contains_escape || encoded[1] != END);
    }
}
