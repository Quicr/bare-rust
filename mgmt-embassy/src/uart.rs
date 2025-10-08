// UART routing and stream management

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxPath {
    None,
    Usb,
    Ui,
    Net,
    UiNet,
    Internal,
}

// Buffer sizes from C code
pub const NET_UART_BUFF_SZ: usize = 2048;
pub const USB_UART_BUFF_SZ: usize = 2048;
pub const UI_UART_BUFF_SZ: usize = 1024;
pub const INTERNAL_BUFF_SZ: usize = 64;
pub const PACKET_SZ: usize = 64;
pub const COMMAND_TIMEOUT_MS: u64 = 1000;
pub const TRANSMISSION_TIMEOUT_MS: u64 = 10000;

// TODO: Implement UartStream and routing logic
