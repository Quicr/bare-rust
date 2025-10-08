// UART routing and stream management

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

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

// DMA buffer size for ring buffer
pub const DMA_BUFFER_SIZE: usize = 1024;

/// Message type for sending data between UART tasks
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum UartMessage {
    /// Data to be transmitted
    Data(heapless::Vec<u8, 256>),
    /// Single byte response (e.g., OK, Ready)
    SingleByte(u8),
}

/// TX channels for routing data between UARTs
pub struct TxChannels {
    pub usb: Channel<CriticalSectionRawMutex, UartMessage, 4>,
    pub ui: Channel<CriticalSectionRawMutex, UartMessage, 4>,
    pub net: Channel<CriticalSectionRawMutex, UartMessage, 4>,
    pub internal: Channel<CriticalSectionRawMutex, UartMessage, 8>,
}

impl TxChannels {
    pub const fn new() -> Self {
        Self {
            usb: Channel::new(),
            ui: Channel::new(),
            net: Channel::new(),
            internal: Channel::new(),
        }
    }
}

impl Default for TxChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Routing configuration for each UART
pub struct UartRouting {
    pub usb_path: TxPath,
    pub ui_path: TxPath,
    pub net_path: TxPath,
}

impl UartRouting {
    pub const fn new() -> Self {
        Self {
            usb_path: TxPath::Internal, // USB defaults to internal (command parsing)
            ui_path: TxPath::None,
            net_path: TxPath::None,
        }
    }
}

impl Default for UartRouting {
    fn default() -> Self {
        Self::new()
    }
}

// Response bytes
pub const OK_BYTE: u8 = 0x80;
pub const READY_BYTE: u8 = 0x81;
pub const OK_ASCII: &[u8] = b"Ok\n";

use defmt::*;
use embassy_stm32::usart::{RingBufferedUartRx, UartTx};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

/// Type alias for shared routing configuration
pub type SharedRouting = &'static Mutex<ThreadModeRawMutex, UartRouting>;

/// RX task for a UART - reads data and routes it based on configured path
pub async fn uart_rx_task(
    mut rx: RingBufferedUartRx<'static>,
    channels: &'static TxChannels,
    routing: SharedRouting,
    uart_name: &'static str,
    get_path: impl Fn(&UartRouting) -> TxPath,
) {
    info!("{} RX task started", uart_name);
    let mut buf = [0u8; 64];

    loop {
        match rx.read(&mut buf).await {
            Ok(n) if n > 0 => {
                // Get the current routing path
                let path = {
                    let routing = routing.lock().await;
                    get_path(&routing)
                };

                // Route data based on path
                if let Ok(vec) = heapless::Vec::from_slice(&buf[..n]) {
                    match path {
                        TxPath::None => {
                            // Drop data
                        }
                        TxPath::Usb => {
                            let _ = channels.usb.send(UartMessage::Data(vec)).await;
                        }
                        TxPath::Ui => {
                            let _ = channels.ui.send(UartMessage::Data(vec)).await;
                        }
                        TxPath::Net => {
                            let _ = channels.net.send(UartMessage::Data(vec)).await;
                        }
                        TxPath::UiNet => {
                            let _ = channels.ui.send(UartMessage::Data(vec.clone())).await;
                            let _ = channels.net.send(UartMessage::Data(vec)).await;
                        }
                        TxPath::Internal => {
                            let _ = channels.internal.send(UartMessage::Data(vec)).await;
                        }
                    }
                }
            }
            Ok(_) => {
                // No data read
            }
            Err(e) => {
                error!("{} RX error: {:?}", uart_name, e);
            }
        }
    }
}

/// TX task for a UART - receives data from channel and transmits it
pub async fn uart_tx_task<const N: usize>(
    mut tx: UartTx<'static, embassy_stm32::mode::Async>,
    channel: &'static Channel<CriticalSectionRawMutex, UartMessage, N>,
    uart_name: &'static str,
) {
    info!("{} TX task started", uart_name);

    loop {
        let msg = channel.receive().await;

        match msg {
            UartMessage::Data(vec) => {
                if let Err(e) = tx.write(&vec).await {
                    error!("{} TX error: {:?}", uart_name, e);
                }
            }
            UartMessage::SingleByte(byte) => {
                if let Err(e) = tx.write(&[byte]).await {
                    error!("{} TX error: {:?}", uart_name, e);
                }
            }
        }
    }
}
