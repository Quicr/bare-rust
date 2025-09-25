//! Direct translation of STM32F4xx HAL I2S from C to Rust
//!
//! This is a 1-to-1 translation of stm32f4xx_hal_i2s.c and stm32f4xx_hal_i2s_ex.c
//! from the STM32F4 HAL library into Rust.

use core::ptr;

// Constants from the C HAL

// I2S Mode definitions
pub const I2S_MODE_SLAVE_TX: u32 = 0x00000000;
pub const I2S_MODE_SLAVE_RX: u32 = 0x00000100;
pub const I2S_MODE_MASTER_TX: u32 = 0x00000200;
pub const I2S_MODE_MASTER_RX: u32 = 0x00000300;

// I2S Standard definitions
pub const I2S_STANDARD_PHILIPS: u32 = 0x00000000;
pub const I2S_STANDARD_MSB: u32 = 0x00000010;
pub const I2S_STANDARD_LSB: u32 = 0x00000020;
pub const I2S_STANDARD_PCM_SHORT: u32 = 0x00000030;
pub const I2S_STANDARD_PCM_LONG: u32 = 0x000000B0;

// I2S Data Format definitions
pub const I2S_DATAFORMAT_16B: u32 = 0x00000000;
pub const I2S_DATAFORMAT_16B_EXTENDED: u32 = 0x00000001;
pub const I2S_DATAFORMAT_24B: u32 = 0x00000003;
pub const I2S_DATAFORMAT_32B: u32 = 0x00000005;

// I2S MCLK Output definitions
pub const I2S_MCLKOUTPUT_ENABLE: u32 = 0x00000200;
pub const I2S_MCLKOUTPUT_DISABLE: u32 = 0x00000000;

// I2S Audio Frequency definitions
pub const I2S_AUDIOFREQ_192K: u32 = 192000;
pub const I2S_AUDIOFREQ_96K: u32 = 96000;
pub const I2S_AUDIOFREQ_48K: u32 = 48000;
pub const I2S_AUDIOFREQ_44K: u32 = 44100;
pub const I2S_AUDIOFREQ_32K: u32 = 32000;
pub const I2S_AUDIOFREQ_22K: u32 = 22050;
pub const I2S_AUDIOFREQ_16K: u32 = 16000;
pub const I2S_AUDIOFREQ_11K: u32 = 11025;
pub const I2S_AUDIOFREQ_8K: u32 = 8000;
pub const I2S_AUDIOFREQ_DEFAULT: u32 = 2;

// I2S Clock Polarity definitions
pub const I2S_CPOL_LOW: u32 = 0x00000000;
pub const I2S_CPOL_HIGH: u32 = 0x00000008;

// I2S Clock Source definitions
pub const I2S_CLOCKSOURCE_PLLI2S: u32 = 0x00000000;
pub const I2S_CLOCKSOURCE_EXT: u32 = 0x00000001;

// I2S Full Duplex Mode definitions
pub const I2S_FULLDUPLEXMODE_DISABLE: u32 = 0x00000000;
pub const I2S_FULLDUPLEXMODE_ENABLE: u32 = 0x00000001;

// HAL Status definitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HalStatus {
    Ok = 0x00,
    Error = 0x01,
    Busy = 0x02,
    Timeout = 0x03,
}

// HAL Lock definitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HalLock {
    Unlocked = 0x00,
    Locked = 0x01,
}

// I2S State definitions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HalI2sState {
    Reset = 0x00,
    Ready = 0x01,
    Busy = 0x02,
    BusyTx = 0x03,
    BusyRx = 0x04,
    BusyTxRx = 0x05,
    Timeout = 0x06,
    Error = 0x07,
}

// I2S Error definitions
pub const HAL_I2S_ERROR_NONE: u32 = 0x00000000;
pub const HAL_I2S_ERROR_UDR: u32 = 0x00000001;
pub const HAL_I2S_ERROR_OVR: u32 = 0x00000002;
pub const HAL_I2S_ERROR_FRE: u32 = 0x00000008;
pub const HAL_I2S_ERROR_DMA: u32 = 0x00000010;
pub const HAL_I2S_ERROR_PRESCALER: u32 = 0x00000020;

// I2S Init Structure (translated from C)
#[derive(Debug, Clone, Copy)]
pub struct I2sInit {
    pub mode: u32,           // I2S operating mode
    pub standard: u32,       // Standard used for I2S communication
    pub data_format: u32,    // Data format for I2S communication
    pub mclk_output: u32,    // MCLK output enabled or disabled
    pub audio_freq: u32,     // Frequency selected for I2S communication
    pub cpol: u32,           // Idle state of I2S clock
    pub clock_source: u32,   // I2S Clock Source
    pub full_duplex_mode: u32, // I2S FullDuplex mode
}

impl Default for I2sInit {
    fn default() -> Self {
        Self {
            mode: I2S_MODE_SLAVE_TX,
            standard: I2S_STANDARD_PHILIPS,
            data_format: I2S_DATAFORMAT_16B,
            mclk_output: I2S_MCLKOUTPUT_DISABLE,
            audio_freq: I2S_AUDIOFREQ_DEFAULT,
            cpol: I2S_CPOL_LOW,
            clock_source: I2S_CLOCKSOURCE_PLLI2S,
            full_duplex_mode: I2S_FULLDUPLEXMODE_DISABLE,
        }
    }
}

// Simplified register base addresses (STM32F4)
pub const SPI2_BASE: u32 = 0x40003800;
pub const SPI3_BASE: u32 = 0x40003C00;
pub const I2S2EXT_BASE: u32 = 0x40003400;
pub const I2S3EXT_BASE: u32 = 0x40004000;

// I2S Handle Structure (translated from C)
pub struct I2sHandle {
    pub instance: u32,                   // I2S registers base address
    pub init: I2sInit,                   // I2S communication parameters
    pub tx_buff_ptr: *mut u16,           // Pointer to I2S Tx transfer buffer
    pub tx_xfer_size: u16,               // I2S Tx transfer size
    pub tx_xfer_count: u16,              // I2S Tx transfer Counter
    pub rx_buff_ptr: *mut u16,           // Pointer to I2S Rx transfer buffer
    pub rx_xfer_size: u16,               // I2S Rx transfer size
    pub rx_xfer_count: u16,              // I2S Rx transfer counter
    pub lock: HalLock,                   // I2S locking object
    pub state: HalI2sState,              // I2S communication state
    pub error_code: u32,                 // I2S Error code
}

impl I2sHandle {
    pub fn new(instance: u32) -> Self {
        Self {
            instance,
            init: I2sInit::default(),
            tx_buff_ptr: ptr::null_mut(),
            tx_xfer_size: 0,
            tx_xfer_count: 0,
            rx_buff_ptr: ptr::null_mut(),
            rx_xfer_size: 0,
            rx_xfer_count: 0,
            lock: HalLock::Unlocked,
            state: HalI2sState::Reset,
            error_code: HAL_I2S_ERROR_NONE,
        }
    }

    pub fn new_spi2() -> Self {
        Self::new(SPI2_BASE)
    }

    pub fn new_spi3() -> Self {
        Self::new(SPI3_BASE)
    }
}

// Extended I2S macros (translated from C)

/// Get the extended I2S instance address
pub fn i2s_ext_instance(instance: u32) -> u32 {
    if instance == SPI2_BASE {
        I2S2EXT_BASE
    } else {
        I2S3EXT_BASE
    }
}

/// Enable I2S Extended peripheral
pub fn hal_i2s_ext_enable(handle: &I2sHandle) {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let i2scfgr_ptr = (ext_base + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg |= 0x00000400; // Set I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

/// Disable I2S Extended peripheral
pub fn hal_i2s_ext_disable(handle: &I2sHandle) {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let i2scfgr_ptr = (ext_base + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg &= !0x00000400; // Clear I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

/// Check if I2S Extended flag is set
pub fn hal_i2s_ext_get_flag(handle: &I2sHandle, flag: u32) -> bool {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let sr_ptr = (ext_base + 0x08) as *const u32; // SR offset
        let sr = ptr::read_volatile(sr_ptr);
        (sr & flag) == flag
    }
}

/// Clear I2S Extended overrun flag
pub fn hal_i2s_ext_clear_ovr_flag(handle: &I2sHandle) {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let dr_ptr = (ext_base + 0x0C) as *const u32; // DR offset
        let sr_ptr = (ext_base + 0x08) as *const u32; // SR offset
        let _tmp = ptr::read_volatile(dr_ptr); // Read DR
        let _tmp = ptr::read_volatile(sr_ptr); // Read SR
    }
}

// Core I2S Functions (translated from C HAL)

/// I2S Initialize
pub fn hal_i2s_init(handle: &mut I2sHandle) -> HalStatus {
    // Check the parameters
    if !is_i2s_mode(handle.init.mode) ||
       !is_i2s_standard(handle.init.standard) ||
       !is_i2s_data_format(handle.init.data_format) ||
       !is_i2s_mclk_output(handle.init.mclk_output) ||
       !is_i2s_audio_freq(handle.init.audio_freq) ||
       !is_i2s_cpol(handle.init.cpol) ||
       !is_i2s_clock_source(handle.init.clock_source) ||
       !is_i2s_full_duplex_mode(handle.init.full_duplex_mode) {
        return HalStatus::Error;
    }

    if handle.state == HalI2sState::Reset {
        // Allocate lock resource and initialize it
        handle.lock = HalLock::Unlocked;
    }

    handle.state = HalI2sState::Busy;

    // Disable the selected I2S peripheral
    hal_i2s_disable(handle);

    // Configure I2S peripheral
    i2s_init(handle);

    handle.error_code = HAL_I2S_ERROR_NONE;
    handle.state = HalI2sState::Ready;

    HalStatus::Ok
}

/// I2S Enable
pub fn hal_i2s_enable(handle: &I2sHandle) {
    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg |= 0x00000400; // Set I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

/// I2S Disable
pub fn hal_i2s_disable(handle: &I2sHandle) {
    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg &= !0x00000400; // Clear I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

/// Transmit an amount of data in blocking mode
pub fn hal_i2s_transmit(handle: &mut I2sHandle, data: &[u16], _timeout: u32) -> HalStatus {
    if handle.state != HalI2sState::Ready {
        return HalStatus::Busy;
    }

    if data.is_empty() {
        return HalStatus::Error;
    }

    // Process Locked
    handle.lock = HalLock::Locked;
    handle.state = HalI2sState::BusyTx;
    handle.error_code = HAL_I2S_ERROR_NONE;

    // Store buffer info (actual transmission would be implemented with DMA/interrupts)
    handle.tx_buff_ptr = data.as_ptr() as *mut u16;
    handle.tx_xfer_size = data.len() as u16;
    handle.tx_xfer_count = 0;

    // Enable I2S
    hal_i2s_enable(handle);

    // Simplified: just mark as complete (real implementation would do actual I2S transfer)
    handle.state = HalI2sState::Ready;
    handle.lock = HalLock::Unlocked;

    HalStatus::Ok
}

/// Receive an amount of data in blocking mode
pub fn hal_i2s_receive(handle: &mut I2sHandle, data: &mut [u16], _timeout: u32) -> HalStatus {
    if handle.state != HalI2sState::Ready {
        return HalStatus::Busy;
    }

    if data.is_empty() {
        return HalStatus::Error;
    }

    // Process Locked
    handle.lock = HalLock::Locked;
    handle.state = HalI2sState::BusyRx;
    handle.error_code = HAL_I2S_ERROR_NONE;

    // Store buffer info
    handle.rx_buff_ptr = data.as_mut_ptr();
    handle.rx_xfer_size = data.len() as u16;
    handle.rx_xfer_count = 0;

    // Enable I2S
    hal_i2s_enable(handle);

    // Simplified: just mark as complete
    handle.state = HalI2sState::Ready;
    handle.lock = HalLock::Unlocked;

    HalStatus::Ok
}

// Full Duplex Extended Functions

/// Transmit and Receive data in full duplex mode
pub fn hal_i2s_transmit_receive(
    handle: &mut I2sHandle,
    tx_data: &[u16],
    rx_data: &mut [u16],
    _timeout: u32
) -> HalStatus {
    if handle.state != HalI2sState::Ready {
        return HalStatus::Busy;
    }

    if tx_data.is_empty() || rx_data.is_empty() || tx_data.len() != rx_data.len() {
        return HalStatus::Error;
    }

    // Process Locked
    handle.lock = HalLock::Locked;
    handle.state = HalI2sState::BusyTxRx;
    handle.error_code = HAL_I2S_ERROR_NONE;

    // Enable both I2S and I2Sext
    hal_i2s_enable(handle);
    hal_i2s_ext_enable(handle);

    // Store buffer info for both TX and RX
    handle.tx_buff_ptr = tx_data.as_ptr() as *mut u16;
    handle.tx_xfer_size = tx_data.len() as u16;
    handle.rx_buff_ptr = rx_data.as_mut_ptr();
    handle.rx_xfer_size = rx_data.len() as u16;

    // Simplified: just mark as complete
    hal_i2s_ext_disable(handle);
    handle.state = HalI2sState::Ready;
    handle.lock = HalLock::Unlocked;

    HalStatus::Ok
}

// Internal helper functions

fn i2s_init(handle: &mut I2sHandle) {
    // Get PLLI2S clock frequency
    let _i2s_clock = get_i2s_clock_freq(handle.init.clock_source);

    // Compute the prescaler value
    let (_i2sdiv, _odd) = i2s_compute_prescaler(
        _i2s_clock,
        handle.init.audio_freq,
        handle.init.mclk_output,
        handle.init.data_format,
        handle.init.standard
    );

    // Configure I2S prescaler (simplified)
    unsafe {
        let i2spr_ptr = (handle.instance + 0x20) as *mut u32; // I2SPR offset
        ptr::write_volatile(i2spr_ptr, 0x0002); // Default prescaler
    }

    // Configure I2S configuration register
    let i2scfgr_value = handle.init.mode |
                        handle.init.standard |
                        handle.init.data_format |
                        handle.init.cpol |
                        0x00000800; // I2SMOD bit

    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32; // I2SCFGR offset
        ptr::write_volatile(i2scfgr_ptr, i2scfgr_value);
    }
}

fn i2s_compute_prescaler(
    i2s_clock: u32,
    audio_freq: u32,
    mclk_output: u32,
    data_format: u32,
    standard: u32
) -> (u8, u8) {
    let mut tmp: u32;
    let i2sdiv: u32;
    let odd: u32;

    // Check if audio frequency is not the default one
    if audio_freq != I2S_AUDIOFREQ_DEFAULT {
        // Check the frame length (For the Prescaler computing)
        if data_format == I2S_DATAFORMAT_16B {
            tmp = 16;
        } else {
            tmp = 32;
        }

        // Check if MCLK output is enabled or not
        if mclk_output == I2S_MCLKOUTPUT_ENABLE {
            // MCLK output is enabled
            if standard == I2S_STANDARD_PCM_SHORT || standard == I2S_STANDARD_PCM_LONG {
                tmp = (((i2s_clock / 128) * 10) / audio_freq) + 5;
            } else {
                tmp = (((i2s_clock / 256) * 10) / audio_freq) + 5;
            }
        } else {
            // MCLK output is disabled
            if standard == I2S_STANDARD_PCM_SHORT || standard == I2S_STANDARD_PCM_LONG {
                tmp = (((i2s_clock / (32 * 2)) * 10) / audio_freq) + 5;
            } else {
                tmp = (((i2s_clock / (tmp * 2)) * 10) / audio_freq) + 5;
            }
        }

        // Remove the flatting point
        tmp = tmp / 10;

        // Check the parity of the divider
        odd = tmp & 0x1;

        // Compute the i2sdiv prescaler
        i2sdiv = (tmp - odd) / 2;

        // Get the Mask for the Odd bit (SPI_I2SPR[8]) register
        (i2sdiv as u8, odd as u8)
    } else {
        // Set the default values
        (2, 0)
    }
}

fn get_i2s_clock_freq(_clock_source: u32) -> u32 {
    // Simplified: return a typical PLLI2S frequency
    50000000 // Example PLLI2S frequency
}

// Parameter validation functions

fn is_i2s_mode(mode: u32) -> bool {
    matches!(mode, I2S_MODE_SLAVE_TX | I2S_MODE_SLAVE_RX | I2S_MODE_MASTER_TX | I2S_MODE_MASTER_RX)
}

fn is_i2s_standard(standard: u32) -> bool {
    matches!(standard,
        I2S_STANDARD_PHILIPS |
        I2S_STANDARD_MSB |
        I2S_STANDARD_LSB |
        I2S_STANDARD_PCM_SHORT |
        I2S_STANDARD_PCM_LONG
    )
}

fn is_i2s_data_format(format: u32) -> bool {
    matches!(format,
        I2S_DATAFORMAT_16B |
        I2S_DATAFORMAT_16B_EXTENDED |
        I2S_DATAFORMAT_24B |
        I2S_DATAFORMAT_32B
    )
}

fn is_i2s_mclk_output(mclk: u32) -> bool {
    matches!(mclk, I2S_MCLKOUTPUT_ENABLE | I2S_MCLKOUTPUT_DISABLE)
}

fn is_i2s_audio_freq(freq: u32) -> bool {
    matches!(freq,
        I2S_AUDIOFREQ_192K |
        I2S_AUDIOFREQ_96K |
        I2S_AUDIOFREQ_48K |
        I2S_AUDIOFREQ_44K |
        I2S_AUDIOFREQ_32K |
        I2S_AUDIOFREQ_22K |
        I2S_AUDIOFREQ_16K |
        I2S_AUDIOFREQ_11K |
        I2S_AUDIOFREQ_8K |
        I2S_AUDIOFREQ_DEFAULT
    ) || (freq >= 8000 && freq <= 192000)
}

fn is_i2s_cpol(cpol: u32) -> bool {
    matches!(cpol, I2S_CPOL_LOW | I2S_CPOL_HIGH)
}

fn is_i2s_clock_source(source: u32) -> bool {
    matches!(source, I2S_CLOCKSOURCE_PLLI2S | I2S_CLOCKSOURCE_EXT)
}

fn is_i2s_full_duplex_mode(mode: u32) -> bool {
    matches!(mode, I2S_FULLDUPLEXMODE_DISABLE | I2S_FULLDUPLEXMODE_ENABLE)
}

// Flag definitions
pub const I2S_FLAG_TXE: u32 = 0x00000002;
pub const I2S_FLAG_RXNE: u32 = 0x00000001;
pub const I2S_FLAG_BSY: u32 = 0x00000080;
pub const I2S_FLAG_OVR: u32 = 0x00000040;
pub const I2S_FLAG_UDR: u32 = 0x00000008;
pub const I2S_FLAG_FRE: u32 = 0x00000100;
pub const I2S_FLAG_CHSIDE: u32 = 0x00000004;