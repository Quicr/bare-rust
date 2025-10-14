use thiserror::Error;

#[derive(Error, Debug)]
pub enum HactarError {
    #[error("Serial port error: {0}")]
    SerialPort(#[from] serialport::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("No Hactar devices found on any serial port. Please check connections and try again.")]
    NoDevicesFound,

    #[error("Device did not respond within timeout period. Check that device is powered on and in bootloader mode.")]
    NoResponse,

    #[error("Device rejected command (NACK). The chip may not support this operation or may not be in the correct mode.")]
    Nack,

    #[error("Failed to sync with bootloader. Ensure device is in bootloader mode and no other program is using the serial port.")]
    SyncFailed,

    #[error("Invalid pattern received: expected {expected:#x}, got {got:#x}")]
    InvalidPattern { expected: u8, got: u8 },

    #[error("Invalid handshake: expected {expected:#x}, got {got:#x}")]
    InvalidHandshake { expected: u8, got: u8 },

    #[error("Chip ID {0:#x} not found in configuration. This may be an unsupported or unrecognized chip variant.")]
    UnknownChipId(u16),

    #[error("Unsupported chip: {0}. Valid options are: mgmt, ui, net")]
    UnsupportedChip(String),

    #[error("Flash verification failed at address {0:#x}. Data written does not match data read back.")]
    VerificationFailed(u32),

    #[error("Erase verification failed at sector {0}. Sector is not empty after erase operation.")]
    EraseVerificationFailed(usize),

    #[error("Memory write failed at address {0:#x}. Device may have rejected the write operation.")]
    WriteFailed(u32),

    #[error("Memory read failed at address {0:#x}. Address may be invalid or inaccessible.")]
    ReadFailed(u32),

    #[error("Binary is too large for available flash memory. Try a smaller binary or check chip configuration.")]
    InsufficientFlash,

    #[error("Binary file not found: {0}. Please check the file path and try again.")]
    BinaryNotFound(String),

    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("MD5 checksum mismatch. Firmware in flash does not match the source file. Flash may be corrupted.")]
    Md5Mismatch,

    #[error("SLIP packet error: {0}. Communication protocol error with ESP32 bootloader.")]
    SlipPacket(String),

    #[error("ESP32 flash operation failed. Device may have rejected the flash command or encountered an internal error.")]
    Esp32FlashError,

    #[error("Invalid command. Use --help to see available commands.")]
    InvalidCommand,

    #[error("Port selection cancelled by user.")]
    PortSelectionCancelled,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, HactarError>;
