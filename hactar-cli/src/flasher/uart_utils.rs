// UART utility functions for low-level serial communication

use crate::utility::errors::{HactarError, Result};
use serialport::SerialPort;
use std::io::{Read, Write};

pub const ACK: u8 = 0x79;
pub const NACK: u8 = 0x1F;
pub const OK: u8 = 0x80;
pub const READY: u8 = 0x81;
pub const MAX_WAIT: usize = 15;

/// Write a single byte to UART
pub fn write_byte(port: &mut Box<dyn SerialPort>, byte: u8, complement: bool) -> Result<()> {
    let data = if complement {
        vec![byte, byte ^ 0xFF]
    } else {
        vec![byte]
    };
    port.write_all(&data)?;
    Ok(())
}

/// Write a byte and wait for ACK response
pub fn write_byte_wait_for_ack(
    port: &mut Box<dyn SerialPort>,
    byte: u8,
    retry_num: usize,
    complement: bool,
) -> Result<u8> {
    let data = if complement {
        vec![byte, byte ^ 0xFF]
    } else {
        vec![byte]
    };

    for _ in 0..retry_num {
        port.write_all(&data)?;

        if let Ok(reply) = get_bytes(port, 1) {
            if reply == ACK {
                return Ok(reply);
            }
        }
    }

    Err(HactarError::NoResponse)
}

/// Write multiple bytes and wait for ACK response
pub fn write_bytes_wait_for_ack(
    port: &mut Box<dyn SerialPort>,
    bytes: &[u8],
    retry_num: usize,
) -> Result<u8> {
    for _ in 0..retry_num {
        port.write_all(bytes)?;

        if let Ok(reply) = get_bytes(port, 1) {
            return Ok(reply);
        }
    }

    Err(HactarError::NoResponse)
}

/// Read bytes from UART, returns error code on no response
pub fn get_bytes(port: &mut Box<dyn SerialPort>, num_bytes: usize) -> Result<u8> {
    let mut buf = vec![0u8; num_bytes];
    let bytes_read = port.read(&mut buf)?;

    if bytes_read < 1 {
        return Err(HactarError::NoResponse);
    }

    if num_bytes == 1 {
        Ok(buf[0])
    } else {
        // For multiple bytes, return the first byte
        // (Python version returns array for >1 bytes, but for ACK checking we typically need single byte)
        Ok(buf[0])
    }
}

/// Read bytes from UART, throws error on no response
pub fn try_get_bytes(port: &mut Box<dyn SerialPort>, num_bytes: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; num_bytes];
    let bytes_read = port.read(&mut buf)?;

    if bytes_read < num_bytes {
        return Err(HactarError::NoResponse);
    }

    Ok(buf)
}

/// Try to receive a specific pattern
pub fn try_pattern(
    port: &mut Box<dyn SerialPort>,
    pattern: u8,
    recv_bytes_cnt: usize,
    num_retry: usize,
) -> Result<()> {
    for _ in 0..num_retry {
        if let Ok(rx) = get_bytes(port, recv_bytes_cnt) {
            if rx == pattern {
                return Ok(());
            }
        }
    }

    Err(HactarError::InvalidPattern {
        expected: pattern,
        got: 0, // We don't know what we got
    })
}

/// Try to complete a handshake by receiving pattern and echoing it back
pub fn try_handshake(
    port: &mut Box<dyn SerialPort>,
    pattern: u8,
    recv_bytes_cnt: usize,
    num_retry: usize,
) -> Result<()> {
    for _ in 0..num_retry {
        if let Ok(rx) = get_bytes(port, recv_bytes_cnt) {
            if rx == pattern {
                write_byte(port, pattern, false)?;
                return Ok(());
            }
        }
    }

    Err(HactarError::InvalidHandshake {
        expected: pattern,
        got: 0, // We don't know what we got
    })
}
