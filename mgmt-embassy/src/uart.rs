// UART routing and stream management

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxPath {
    None,
    Usb,
    Ui,
    Net,
    Internal,
}

// DMA buffer size for ring buffer
pub const DMA_BUFFER_SIZE: usize = 1024;

/// Routing configuration for each UART
pub struct UartRouting {
    pub usb_path: TxPath,
    pub ui_path: TxPath,
    pub net_path: TxPath,
}

impl Default for UartRouting {
    fn default() -> Self {
        Self {
            usb_path: TxPath::Internal, // USB defaults to internal (command parsing)
            ui_path: TxPath::None,
            net_path: TxPath::None,
        }
    }
}

// Response bytes
pub const OK_BYTE: u8 = 0x80;
pub const READY_BYTE: u8 = 0x81;
pub const OK_ASCII: &[u8] = b"Ok\n";
