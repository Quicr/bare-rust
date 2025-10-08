// Command definitions and handlers

use defmt::*;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;

use crate::{
    gpio::{NetControl, UiControl},
    state::{State, DEFAULT_STATE},
    uart::{TxPath, UartRouting, OK_ASCII},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
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

impl Command {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Command::Version),
            1 => Some(Command::WhoAreYou),
            2 => Some(Command::HardReset),
            3 => Some(Command::Reset),
            4 => Some(Command::ResetUi),
            5 => Some(Command::ResetNet),
            6 => Some(Command::FlashUi),
            7 => Some(Command::FlashNet),
            8 => Some(Command::EnableLogs),
            9 => Some(Command::EnableLogsUi),
            10 => Some(Command::EnableLogsNet),
            11 => Some(Command::DisableLogs),
            12 => Some(Command::DisableLogsUi),
            13 => Some(Command::DisableLogsNet),
            14 => Some(Command::DefaultLogging),
            15 => Some(Command::ToUi),
            16 => Some(Command::ToNet),
            17 => Some(Command::Loopback),
            _ => None,
        }
    }
}

pub const CMD_COUNT: usize = 18;

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

                        if let Some(command) = Command::from_u8(cmd_byte) {
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
    pub routing: &'static Mutex<ThreadModeRawMutex, UartRouting>,
    pub ui_control: &'static Mutex<ThreadModeRawMutex, Option<UiControl>>,
    pub net_control: &'static Mutex<ThreadModeRawMutex, Option<NetControl>>,
    pub state: &'static Mutex<ThreadModeRawMutex, State>,
}

/// Command handlers
impl CommandContext {
    pub async fn handle_version(&self) -> &'static [u8] {
        // TODO: Get actual version
        b"v1.0.0\n"
    }

    pub async fn handle_who_are_you(&self) -> &'static [u8] {
        b"HELLO, I AM A HACTAR DEVICE"
    }

    pub async fn handle_hard_reset(&self) {
        info!("Hard reset requested");
        let mut state = self.state.lock().await;
        *state = DEFAULT_STATE;
    }

    pub async fn handle_reset(&self) {
        self.handle_reset_ui().await;
        self.handle_reset_net().await;
    }

    pub async fn handle_reset_ui(&self) {
        info!("Resetting UI chip");
        let mut ui_control = self.ui_control.lock().await;
        if let Some(ref mut ctrl) = *ui_control {
            ctrl.normal_mode().await;
        }
    }

    pub async fn handle_reset_net(&self) {
        info!("Resetting NET chip");
        let mut net_control = self.net_control.lock().await;
        if let Some(ref mut ctrl) = *net_control {
            ctrl.normal_mode().await;
        }
    }

    pub async fn handle_flash_ui(&self) {
        info!("Entering UI flash mode");

        // Hold NET in reset
        {
            let mut net_control = self.net_control.lock().await;
            if let Some(ref mut ctrl) = *net_control {
                ctrl.hold_in_reset().await;
            }
        }

        // Configure routing: USB->UI, UI->USB, NET->None
        {
            let mut routing = self.routing.lock().await;
            routing.usb_path = TxPath::Ui;
            routing.ui_path = TxPath::Usb;
            routing.net_path = TxPath::None;
        }

        // TODO: Send OK byte and reconfigure UART to 9E1
        // TODO: Put UI into bootloader mode
        // TODO: Send Ready byte
    }

    pub async fn handle_flash_net(&self) {
        info!("Entering NET flash mode");

        // Hold UI in reset
        {
            let mut ui_control = self.ui_control.lock().await;
            if let Some(ref mut ctrl) = *ui_control {
                ctrl.hold_in_reset().await;
            }
        }

        // Configure routing: USB->NET, NET->USB, UI->None
        {
            let mut routing = self.routing.lock().await;
            routing.usb_path = TxPath::Net;
            routing.net_path = TxPath::Usb;
            routing.ui_path = TxPath::None;
        }

        // TODO: Send OK byte
        // TODO: Put NET into bootloader mode
        // TODO: Send Ready byte
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
        let state = self.state.lock().await;
        let mut routing = self.routing.lock().await;

        match *state {
            State::Normal => {
                routing.ui_path = TxPath::None;
                routing.net_path = TxPath::None;
            }
            State::Debug => {
                routing.ui_path = TxPath::Usb;
                routing.net_path = TxPath::Usb;
            }
            _ => {}
        }
    }

    pub async fn execute(&self, command: Command, _data: &[u8]) -> Option<&'static [u8]> {
        match command {
            Command::Version => Some(self.handle_version().await),
            Command::WhoAreYou => Some(self.handle_who_are_you().await),
            Command::HardReset => {
                self.handle_hard_reset().await;
                Some(OK_ASCII)
            }
            Command::Reset => {
                self.handle_reset().await;
                Some(OK_ASCII)
            }
            Command::ResetUi => {
                self.handle_reset_ui().await;
                Some(OK_ASCII)
            }
            Command::ResetNet => {
                self.handle_reset_net().await;
                Some(OK_ASCII)
            }
            Command::FlashUi => {
                self.handle_flash_ui().await;
                None // Special handling needed
            }
            Command::FlashNet => {
                self.handle_flash_net().await;
                None // Special handling needed
            }
            Command::EnableLogs => {
                self.handle_enable_logs().await;
                Some(OK_ASCII)
            }
            Command::EnableLogsUi => {
                self.handle_enable_logs_ui().await;
                Some(OK_ASCII)
            }
            Command::EnableLogsNet => {
                self.handle_enable_logs_net().await;
                Some(OK_ASCII)
            }
            Command::DisableLogs => {
                self.handle_disable_logs().await;
                Some(OK_ASCII)
            }
            Command::DisableLogsUi => {
                self.handle_disable_logs_ui().await;
                Some(OK_ASCII)
            }
            Command::DisableLogsNet => {
                self.handle_disable_logs_net().await;
                Some(OK_ASCII)
            }
            Command::DefaultLogging => {
                self.handle_default_logging().await;
                Some(OK_ASCII)
            }
            // Data forwarding commands - handled separately
            Command::ToUi | Command::ToNet | Command::Loopback => None,
        }
    }
}
