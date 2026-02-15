use num_enum::TryFromPrimitive;

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
    ToUsb = 17,
}

pub const VERSION: &[u8] = b"v1.0.0\n";
pub const HELLO_I_AM_A_HACTAR_DEVICE: &[u8] = b"HELLO, I AM A HACTAR DEVICE";
pub const OK_ASCII: &[u8] = b"Ok\n";
pub const OK_BYTE: u8 = 0x80;
pub const READY_BYTE: u8 = 0x81;
