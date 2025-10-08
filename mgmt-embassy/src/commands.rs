// Command definitions and handlers

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
