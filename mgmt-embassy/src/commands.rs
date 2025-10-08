// Command definitions and handlers

use defmt::*;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
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

/// Response from command execution
#[derive(Debug)]
pub enum CommandResponse<'a> {
    /// Send data response
    Data(&'a [u8]),
    /// Enter UI flash mode
    FlashUi,
    /// Enter NET flash mode
    FlashNet,
    /// Forward data to UI UART
    ForwardToUi(&'a [u8]),
    /// Forward data to NET UART
    ForwardToNet(&'a [u8]),
    /// Loopback data to USB UART
    Loopback(&'a [u8]),
}

/// TLV packet parser state
#[derive(Debug)]
pub enum ParserState {
    WaitingForHeader,
    ReadingData { command: Command, remaining: u32 },
}

/// TLV parser for command packets
/// Format: [Command: 1 byte][Length: 4 bytes LE][Data: N bytes]
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
                                self.state = ParserState::ReadingData {
                                    command,
                                    remaining: length,
                                };
                                self.header_buf.clear();
                            } else {
                                // Data too large, skip this packet
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
                ParserState::ReadingData { command, remaining } => {
                    if self.data_buf.push(byte).is_err() {
                        // Buffer overflow
                        error!("Data buffer overflow");
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
pub struct CommandContext {
    // Mutex is needed even with single task for:
    // 1. Interior mutability of statics
    // 2. Send/Sync safety across async await points
    pub routing: &'static Mutex<ThreadModeRawMutex, UartRouting>,
    // Option is needed because static initialization can't call new() with peripherals
    // Peripherals are only available after embassy_stm32::init() in main()
    // After initialization in main, these are always Some
    pub ui_control: &'static Mutex<ThreadModeRawMutex, Option<UiControl>>,
    pub net_control: &'static Mutex<ThreadModeRawMutex, Option<NetControl>>,
}

/// Command handlers
impl CommandContext {
    pub async fn handle_version(&self) -> &'static [u8] {
        VERSION
    }

    pub async fn handle_who_are_you(&self) -> &'static [u8] {
        b"HELLO, I AM A HACTAR DEVICE"
    }

    pub async fn handle_hard_reset(&self) {
        info!("Hard reset requested");
        // Reset both chips
        self.handle_reset().await;
        // Reset routing to defaults (Debug mode: logs enabled)
        let mut routing = self.routing.lock().await;
        routing.usb_path = TxPath::Internal;
        routing.ui_path = TxPath::Usb;
        routing.net_path = TxPath::Usb;
    }

    pub async fn handle_reset(&self) {
        self.handle_reset_ui().await;
        self.handle_reset_net().await;
    }

    pub async fn handle_reset_ui(&self) {
        info!("Resetting UI chip");
        let mut ui_control = self.ui_control.lock().await;
        if let Some(ref mut ctrl) = *ui_control {
            ctrl.normal_mode();
        }
    }

    pub async fn handle_reset_net(&self) {
        info!("Resetting NET chip");
        let mut net_control = self.net_control.lock().await;
        if let Some(ref mut ctrl) = *net_control {
            ctrl.normal_mode();
        }
    }

    pub async fn handle_flash_ui(&self) {
        info!("Entering UI flash mode");

        // Hold NET in reset
        {
            let mut net_control = self.net_control.lock().await;
            if let Some(ref mut ctrl) = *net_control {
                ctrl.hold_in_reset();
            }
        }

        // Configure routing: USB->UI, UI->USB, NET->None
        {
            let mut routing = self.routing.lock().await;
            routing.usb_path = TxPath::Ui;
            routing.ui_path = TxPath::Usb;
            routing.net_path = TxPath::None;
        }

        // Flash mode sequence handled in main loop
    }

    pub async fn handle_flash_net(&self) {
        info!("Entering NET flash mode");

        // Hold UI in reset
        {
            let mut ui_control = self.ui_control.lock().await;
            if let Some(ref mut ctrl) = *ui_control {
                ctrl.hold_in_reset();
            }
        }

        // Configure routing: USB->NET, NET->USB, UI->None
        {
            let mut routing = self.routing.lock().await;
            routing.usb_path = TxPath::Net;
            routing.net_path = TxPath::Usb;
            routing.ui_path = TxPath::None;
        }

        // Flash mode sequence handled in main loop
    }

    pub async fn handle_enable_logs(&self) {
        self.handle_enable_logs_ui().await;
        self.handle_enable_logs_net().await;
    }

    pub async fn handle_enable_logs_ui(&self) {
        info!("Enabling UI logs");
        let mut routing = self.routing.lock().await;
        routing.ui_path = TxPath::Usb;
    }

    pub async fn handle_enable_logs_net(&self) {
        info!("Enabling NET logs");
        let mut routing = self.routing.lock().await;
        routing.net_path = TxPath::Usb;
    }

    pub async fn handle_disable_logs(&self) {
        self.handle_disable_logs_ui().await;
        self.handle_disable_logs_net().await;
    }

    pub async fn handle_disable_logs_ui(&self) {
        info!("Disabling UI logs");
        let mut routing = self.routing.lock().await;
        routing.ui_path = TxPath::None;
    }

    pub async fn handle_disable_logs_net(&self) {
        info!("Disabling NET logs");
        let mut routing = self.routing.lock().await;
        routing.net_path = TxPath::None;
    }

    pub async fn handle_default_logging(&self) {
        info!("Setting default logging");
        // Default is Debug mode: enable logs
        let mut routing = self.routing.lock().await;
        routing.ui_path = TxPath::Usb;
        routing.net_path = TxPath::Usb;
    }

    pub async fn execute<'a>(
        &self,
        command: Command,
        data: &'a heapless::Vec<u8, 64>,
    ) -> Option<CommandResponse<'a>> {
        match command {
            Command::Version => Some(CommandResponse::Data(self.handle_version().await)),
            Command::WhoAreYou => Some(CommandResponse::Data(self.handle_who_are_you().await)),
            Command::HardReset => {
                self.handle_hard_reset().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::Reset => {
                self.handle_reset().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::ResetUi => {
                self.handle_reset_ui().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::ResetNet => {
                self.handle_reset_net().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::FlashUi => {
                self.handle_flash_ui().await;
                Some(CommandResponse::FlashUi)
            }
            Command::FlashNet => {
                self.handle_flash_net().await;
                Some(CommandResponse::FlashNet)
            }
            Command::EnableLogs => {
                self.handle_enable_logs().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::EnableLogsUi => {
                self.handle_enable_logs_ui().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::EnableLogsNet => {
                self.handle_enable_logs_net().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::DisableLogs => {
                self.handle_disable_logs().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::DisableLogsUi => {
                self.handle_disable_logs_ui().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::DisableLogsNet => {
                self.handle_disable_logs_net().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            Command::DefaultLogging => {
                self.handle_default_logging().await;
                Some(CommandResponse::Data(OK_ASCII))
            }
            // Data forwarding commands
            Command::ToUi => Some(CommandResponse::ForwardToUi(data.as_slice())),
            Command::ToNet => Some(CommandResponse::ForwardToNet(data.as_slice())),
            Command::Loopback => Some(CommandResponse::Loopback(data.as_slice())),
        }
    }
}
