// Command definitions and handlers

use defmt::*;
use num_enum::TryFromPrimitive;

use crate::{
    gpio::{NetControl, UiControl},
    uart::{TxPath, UartRouting, OK_ASCII},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format, TryFromPrimitive)]
pub enum Command {
    Version = 0,
    WhoAreYou = 1,
    HardReset = 2,
    Reset = 3,
    ResetUi = 4,
    ResetNet = 5,
    FlashUi = 6,
    FlashNet = 7,
    EnableLogs = 8,
    EnableLogsUi = 9,
    EnableLogsNet = 10,
    DisableLogs = 11,
    DisableLogsUi = 12,
    DisableLogsNet = 13,
    DefaultLogging = 14,
    ToUi = 15,
    ToNet = 16,
    Loopback = 17,
}

const VERSION: &[u8] = b"v1.0.0\n";
const HELLO_I_AM_A_HACTAR_DEVICE: &[u8] = b"HELLO, I AM A HACTAR DEVICE";

/// Response from command execution
#[derive(Debug)]
pub enum CommandResponse<'a> {
    /// Enter UI flash mode
    FlashUi,
    /// Enter NET flash mode
    FlashNet,
    /// Send data to UI UART
    ToUi(&'a [u8]),
    /// Send data to NET UART
    ToNet(&'a [u8]),
    /// Send data to USB UART
    ToUsb(&'a [u8]),
}

/// TLV packet parser state
#[derive(Debug)]
pub enum ParserState {
    WaitingForHeader,
    ReadingToUsb { command: Command, remaining: u32 },
}

/// TLV parser for command packets
/// Format: [Command: 1 byte][Length: 4 bytes LE][ToUsb: N bytes]
pub struct TlvParser {
    state: ParserState,
    header_buf: heapless::Vec<u8, 5>,
    data_buf: heapless::Vec<u8, 64>,
}

impl TlvParser {
    pub const fn new() -> Self {
        Self {
            state: ParserState::WaitingForHeader,
            header_buf: heapless::Vec::new(),
            data_buf: heapless::Vec::new(),
        }
    }

    /// Process incoming data, returns Some((command, data)) when a complete packet is parsed
    pub fn process(&mut self, data: &[u8]) -> Option<(Command, heapless::Vec<u8, 64>)> {
        for &byte in data {
            match &mut self.state {
                ParserState::WaitingForHeader => {
                    if self.header_buf.push(byte).is_err() {
                        // Buffer full, we shouldn't get here
                        self.reset();
                        continue;
                    }

                    if self.header_buf.len() == 5 {
                        // Parse header
                        let cmd_byte = self.header_buf[0];
                        let length = u32::from_le_bytes([
                            self.header_buf[1],
                            self.header_buf[2],
                            self.header_buf[3],
                            self.header_buf[4],
                        ]);

                        if let Ok(command) = Command::try_from(cmd_byte) {
                            if length == 0 {
                                // Zero-length command, execute immediately
                                self.reset();
                                return Some((command, heapless::Vec::new()));
                            } else if length <= 64 {
                                // Start reading data
                                self.state = ParserState::ReadingToUsb {
                                    command,
                                    remaining: length,
                                };
                                self.header_buf.clear();
                            } else {
                                // ToUsb too large, skip this packet
                                warn!("Command data too large: {}", length);
                                self.reset();
                            }
                        } else {
                            // Invalid command
                            warn!("Invalid command: {}", cmd_byte);
                            self.reset();
                        }
                    }
                }
                ParserState::ReadingToUsb { command, remaining } => {
                    if self.data_buf.push(byte).is_err() {
                        // Buffer overflow
                        error!("ToUsb buffer overflow");
                        self.reset();
                        continue;
                    }

                    *remaining -= 1;

                    if *remaining == 0 {
                        // Complete packet received
                        let cmd = *command;
                        let data = self.data_buf.clone();
                        self.reset();
                        return Some((cmd, data));
                    }
                }
            }
        }

        None
    }

    fn reset(&mut self) {
        self.state = ParserState::WaitingForHeader;
        self.header_buf.clear();
        self.data_buf.clear();
    }
}

impl Default for TlvParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Command handler context
pub struct CommandContext<'a> {
    pub routing: &'a mut UartRouting,
    pub ui_control: &'a mut UiControl,
    pub net_control: &'a mut NetControl,
}

/// Command handlers
impl<'a> CommandContext<'a> {
    pub async fn reset_ui(&mut self) {
        info!("Resetting UI chip");
        self.ui_control.normal_mode();
    }

    pub async fn reset_net(&mut self) {
        info!("Resetting NET chip");
        self.net_control.normal_mode();
    }

    pub async fn enable_logs_ui(&mut self, enabled: bool) {
        info!("Enabling UI logs");
        self.routing.ui_path = if enabled { TxPath::Usb } else { TxPath::None };
    }

    pub async fn enable_logs_net(&mut self, enabled: bool) {
        info!("Enabling NET logs");
        self.routing.net_path = if enabled { TxPath::Usb } else { TxPath::None };
    }

    pub async fn execute<'b>(
        &mut self,
        command: Command,
        data: &'b heapless::Vec<u8, 64>,
    ) -> Option<CommandResponse<'b>> {
        match command {
            Command::Version => Some(CommandResponse::ToUsb(VERSION)),
            Command::WhoAreYou => Some(CommandResponse::ToUsb(HELLO_I_AM_A_HACTAR_DEVICE)),
            Command::HardReset => {
                info!("Hard reset requested");
                // Reset both chips
                self.reset_ui().await;
                self.reset_net().await;

                // Reset routing to defaults (Debug mode: logs enabled)
                self.routing.usb_path = TxPath::Internal;
                self.routing.ui_path = TxPath::Usb;
                self.routing.net_path = TxPath::Usb;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::Reset => {
                self.reset_ui().await;
                self.reset_net().await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::ResetUi => {
                self.reset_ui().await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::ResetNet => {
                self.reset_net().await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::FlashUi => {
                info!("Entering UI flash mode");

                // Hold NET in reset
                self.net_control.hold_in_reset();

                // Configure routing: USB->UI, UI->USB, NET->None
                self.routing.usb_path = TxPath::Ui;
                self.routing.ui_path = TxPath::Usb;
                self.routing.net_path = TxPath::None;

                // Reconfiguration handled in main loop
                Some(CommandResponse::FlashUi)
            }
            Command::FlashNet => {
                info!("Entering NET flash mode");

                // Hold UI in reset
                self.ui_control.hold_in_reset();

                // Configure routing: USB->NET, NET->USB, UI->None
                self.routing.usb_path = TxPath::Net;
                self.routing.net_path = TxPath::Usb;
                self.routing.ui_path = TxPath::None;

                // Reconfiguration handled in main loop
                Some(CommandResponse::FlashNet)
            }
            Command::EnableLogs => {
                self.enable_logs_ui(true).await;
                self.enable_logs_net(true).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::EnableLogsUi => {
                self.enable_logs_ui(true).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::EnableLogsNet => {
                self.enable_logs_net(true).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::DisableLogs => {
                self.enable_logs_ui(false).await;
                self.enable_logs_net(false).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::DisableLogsUi => {
                self.enable_logs_ui(false).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::DisableLogsNet => {
                self.enable_logs_net(false).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            Command::DefaultLogging => {
                self.enable_logs_ui(true).await;
                self.enable_logs_net(true).await;
                Some(CommandResponse::ToUsb(OK_ASCII))
            }
            // Forwarding commands
            Command::ToUi => Some(CommandResponse::ToUi(data.as_slice())),
            Command::ToNet => Some(CommandResponse::ToNet(data.as_slice())),
            Command::Loopback => Some(CommandResponse::ToUsb(data.as_slice())),
        }
    }
}
