use thiserror::Error;

#[derive(Error, Debug)]
pub enum HactarError {
    #[error("Serial port error: {0}")]
    SerialPort(#[from] serialport::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("No Hactar devices found")]
    NoDevicesFound,

    #[error("Device did not respond")]
    NoResponse,

    #[error("Device sent NACK")]
    Nack,

    #[error("Sync failed")]
    SyncFailed,

    #[error("Invalid pattern received: expected {expected:#x}, got {got:#x}")]
    InvalidPattern { expected: u8, got: u8 },

    #[error("Invalid handshake: expected {expected:#x}, got {got:#x}")]
    InvalidHandshake { expected: u8, got: u8 },

    #[error("Chip ID {0} not found in configuration")]
    UnknownChipId(u16),

    #[error("Unsupported chip: {0}")]
    UnsupportedChip(String),

    #[error("Flash verification failed at address {0:#x}")]
    VerificationFailed(u32),

    #[error("Erase verification failed at sector {0}")]
    EraseVerificationFailed(usize),

    #[error("Memory write failed at address {0:#x}")]
    WriteFailed(u32),

    #[error("Memory read failed at address {0:#x}")]
    ReadFailed(u32),

    #[error("Not enough flash memory for binary")]
    InsufficientFlash,

    #[error("Binary file not found: {0}")]
    BinaryNotFound(String),

    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("MD5 checksum mismatch")]
    Md5Mismatch,

    #[error("SLIP packet error: {0}")]
    SlipPacket(String),

    #[error("ESP32 flash error")]
    Esp32FlashError,

    #[error("Invalid command")]
    InvalidCommand,

    #[error("Port selection cancelled")]
    PortSelectionCancelled,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, HactarError>;
