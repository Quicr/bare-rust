//! # STM32F4xx HAL I2S Driver
//!
//! This module provides a direct Rust translation of the STM32F4xx HAL I2S driver,
//! based on `stm32f4xx_hal_i2s.c` and `stm32f4xx_hal_i2s_ex.c` from ST's official HAL library.
//!
//! ## Features
//!
//! - **Full Duplex I2S**: Support for simultaneous transmit and receive using dual instances
//! - **Extended Instance Support**: I2S2ext/I2S3ext peripheral support for STM32F4 series
//! - **Multiple Standards**: Philips, MSB, LSB, and PCM standards
//! - **Flexible Data Formats**: 16-bit, 24-bit, and 32-bit data formats
//! - **Audio Frequencies**: Support for common audio sample rates (8kHz to 192kHz)
//! - **Memory Safety**: Rust-safe implementation with controlled unsafe blocks
//!
//! ## Architecture
//!
//! The STM32F4 I2S implementation uses a dual-instance approach for full duplex operation:
//! - **Main Instance** (SPI2/SPI3): Handles TX operations and clock generation
//! - **Extended Instance** (I2S2ext/I2S3ext): Handles RX operations in full duplex mode
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! # use crate::hal_i2s::*;
//! // Initialize I2S handle for SPI3
//! let mut i2s = I2sHandle::new_spi3();
//!
//! // Configure I2S parameters
//! i2s.init.mode = I2S_MODE_SLAVE_TX;
//! i2s.init.standard = I2S_STANDARD_PHILIPS;
//! i2s.init.data_format = I2S_DATAFORMAT_16B;
//! i2s.init.audio_freq = 48_000;
//! i2s.init.full_duplex_mode = I2S_FULLDUPLEXMODE_ENABLE;
//!
//! // Initialize the I2S peripheral
//! if hal_i2s_init(&mut i2s) == HalStatus::Ok {
//!     // I2S is ready for use
//! }
//! ```
//!
//! ## Full Duplex Example
//!
//! ```rust,no_run
//! # use crate::hal_i2s::*;
//! # let mut i2s = I2sHandle::new_spi3();
//! # hal_i2s_init(&mut i2s);
//! // Transmit and receive simultaneously
//! let tx_data = [0x1234u16; 100];
//! let mut rx_data = [0u16; 100];
//!
//! match hal_i2s_transmit_receive(&mut i2s, &tx_data, &mut rx_data, 1000) {
//!     HalStatus::Ok => {
//!         // Full duplex transfer completed successfully
//!     }
//!     HalStatus::Timeout => {
//!         // Transfer timed out
//!     }
//!     _ => {
//!         // Handle other errors
//!     }
//! }
//! ```
//!
//! ## Memory Mapping
//!
//! The driver uses direct memory access to STM32F4 registers:
//! - **SPI2**: `0x40003800` (I2S2ext: `0x40003400`)
//! - **SPI3**: `0x40003C00` (I2S3ext: `0x40004000`)
//!
//! ## Safety
//!
//! This module uses unsafe code for direct register access. All unsafe operations are
//! contained within well-defined functions and are safe when used correctly with proper
//! hardware initialization.
use core::ptr;
use defmt::Format;

pub const I2S_MODE_SLAVE_TX: u32 = 0x00000000;

pub const I2S_MODE_SLAVE_RX: u32 = 0x00000100;

pub const I2S_MODE_MASTER_TX: u32 = 0x00000200;

pub const I2S_MODE_MASTER_RX: u32 = 0x00000300;

pub const I2S_STANDARD_PHILIPS: u32 = 0x00000000;

pub const I2S_STANDARD_MSB: u32 = 0x00000010;

pub const I2S_STANDARD_LSB: u32 = 0x00000020;

pub const I2S_STANDARD_PCM_SHORT: u32 = 0x00000030;

pub const I2S_STANDARD_PCM_LONG: u32 = 0x000000B0;

pub const I2S_DATAFORMAT_16B: u32 = 0x00000000;

pub const I2S_DATAFORMAT_16B_EXTENDED: u32 = 0x00000001;

pub const I2S_DATAFORMAT_24B: u32 = 0x00000003;

pub const I2S_DATAFORMAT_32B: u32 = 0x00000005;

pub const I2S_MCLKOUTPUT_ENABLE: u32 = 0x00000200;

pub const I2S_MCLKOUTPUT_DISABLE: u32 = 0x00000000;

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

pub const I2S_CPOL_LOW: u32 = 0x00000000;

pub const I2S_CPOL_HIGH: u32 = 0x00000008;

pub const I2S_CLOCKSOURCE_PLLI2S: u32 = 0x00000000;

pub const I2S_CLOCKSOURCE_EXT: u32 = 0x00000001;

pub const I2S_FULLDUPLEXMODE_DISABLE: u32 = 0x00000000;

pub const I2S_FULLDUPLEXMODE_ENABLE: u32 = 0x00000001;

#[derive(Format, Debug, Clone, Copy, PartialEq)]
pub enum HalStatus {
    Ok = 0x00,

    Error = 0x01,

    Busy = 0x02,

    Timeout = 0x03,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HalLock {
    Unlocked = 0x00,

    Locked = 0x01,
}

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

pub const HAL_I2S_ERROR_NONE: u32 = 0x00000000;

pub const HAL_I2S_ERROR_UDR: u32 = 0x00000001;

pub const HAL_I2S_ERROR_OVR: u32 = 0x00000002;

pub const HAL_I2S_ERROR_FRE: u32 = 0x00000008;

pub const HAL_I2S_ERROR_DMA: u32 = 0x00000010;
pub const HAL_I2S_ERROR_TIMEOUT: u32 = 0x00000020;

pub const HAL_I2S_ERROR_PRESCALER: u32 = 0x00000020;

#[derive(Debug, Clone, Copy)]
pub struct I2sInit {
    pub mode: u32,

    pub standard: u32,

    pub data_format: u32,

    pub mclk_output: u32,

    pub audio_freq: u32,

    pub cpol: u32,

    pub clock_source: u32,

    pub full_duplex_mode: u32,
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

pub struct I2sHandle {
    pub instance: u32,

    pub init: I2sInit,

    pub tx_buff_ptr: *mut u16,

    pub tx_xfer_size: u16,

    pub tx_xfer_count: u16,

    pub rx_buff_ptr: *mut u16,

    pub rx_xfer_size: u16,

    pub rx_xfer_count: u16,

    pub lock: HalLock,

    pub state: HalI2sState,

    pub error_code: u32,
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

pub fn i2s_ext_instance(instance: u32) -> u32 {
    if instance == SPI2_BASE {
        I2S2EXT_BASE
    } else {
        I2S3EXT_BASE
    }
}

pub fn hal_i2s_ext_enable(handle: &I2sHandle) {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let i2scfgr_ptr = (ext_base + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg |= 0x00000400; // Set I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

pub fn hal_i2s_ext_disable(handle: &I2sHandle) {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let i2scfgr_ptr = (ext_base + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg &= !0x00000400; // Clear I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

pub fn hal_i2s_ext_get_flag(handle: &I2sHandle, flag: u32) -> bool {
    let ext_base = i2s_ext_instance(handle.instance);
    unsafe {
        let sr_ptr = (ext_base + 0x08) as *const u32; // SR offset
        let sr = ptr::read_volatile(sr_ptr);
        (sr & flag) == flag
    }
}

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

pub fn hal_i2s_init(handle: &mut I2sHandle) -> HalStatus {
    let i2sdiv: u32;
    let mut i2sodd: u32;
    let mut packetlength: u32;
    let mut tmp: u32;
    let i2sclk: u32;

    // Check the I2S parameters (simplified - assuming all parameters are valid)
    if !is_i2s_mode(handle.init.mode)
        || !is_i2s_standard(handle.init.standard)
        || !is_i2s_data_format(handle.init.data_format)
        || !is_i2s_mclk_output(handle.init.mclk_output)
        || !is_i2s_audio_freq(handle.init.audio_freq)
        || !is_i2s_cpol(handle.init.cpol)
        || !is_i2s_clock_source(handle.init.clock_source)
        || !is_i2s_full_duplex_mode(handle.init.full_duplex_mode)
    {
        return HalStatus::Error;
    }

    if handle.state == HalI2sState::Reset {
        // Allocate lock resource and initialize it
        handle.lock = HalLock::Unlocked;

        // Init the low level hardware: GPIO, CLOCK, NVIC... (already done by hal_i2s_msp_init)
    }

    handle.state = HalI2sState::Busy;

    // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits
    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32;
        let mut i2scfgr = ptr::read_volatile(i2scfgr_ptr);
        i2scfgr &= !(SPI_I2SCFGR_CHLEN
            | SPI_I2SCFGR_DATLEN
            | SPI_I2SCFGR_CKPOL
            | SPI_I2SCFGR_I2SSTD
            | SPI_I2SCFGR_PCMSYNC
            | SPI_I2SCFGR_I2SCFG
            | SPI_I2SCFGR_I2SE
            | SPI_I2SCFGR_I2SMOD);
        ptr::write_volatile(i2scfgr_ptr, i2scfgr);

        // Reset I2SPR register
        let i2spr_ptr = (handle.instance + 0x20) as *mut u32;
        ptr::write_volatile(i2spr_ptr, 0x0002);
    }

    // I2SPR: I2SDIV and ODD Calculation
    // If the requested audio frequency is not the default, compute the prescaler
    if handle.init.audio_freq != I2S_AUDIOFREQ_DEFAULT {
        // Check the frame length (For the Prescaler computing)
        if handle.init.data_format == I2S_DATAFORMAT_16B {
            // Packet length is 16 bits
            packetlength = 16;
        } else {
            // Packet length is 32 bits
            packetlength = 32;
        }

        // I2S standard
        if handle.init.standard <= I2S_STANDARD_LSB {
            // In I2S standard packet length is multiplied by 2
            packetlength = packetlength * 2;
        }

        // Get the source clock value (simplified - use PLLI2S)
        i2sclk = get_i2s_clock_freq(handle.init.clock_source);

        // Compute the Real divider depending on the MCLK output state, with a floating point
        if handle.init.mclk_output == I2S_MCLKOUTPUT_ENABLE {
            // MCLK output is enabled
            tmp = (((i2sclk / 256) * 10) / handle.init.audio_freq) + 5;
        } else {
            // MCLK output is disabled
            tmp = (((i2sclk / packetlength) * 10) / handle.init.audio_freq) + 5;
        }

        // Remove the flatting point
        tmp = tmp / 10;

        // Check the parity of the divider
        i2sodd = tmp & 0x1;

        // Compute the i2sdiv prescaler
        i2sdiv = ((tmp - i2sodd) / 2) & 0xFF;

        // Get the Mask for the Odd bit (SPI_I2SPR[8]) register
        i2sodd = i2sodd << 8;
    } else {
        // Set the default values
        i2sdiv = 2;
        i2sodd = 0;
    }

    // Test if the divider is 1 or 0 or greater than 0xFF
    if (i2sdiv < 2) || (i2sdiv > 0xFF) {
        // Set the error code
        handle.error_code = HAL_I2S_ERROR_PRESCALER;
        return HalStatus::Error;
    }

    // Write to SPIx I2SPR register the computed value
    unsafe {
        let i2spr_ptr = (handle.instance + 0x20) as *mut u32;
        ptr::write_volatile(i2spr_ptr, i2sdiv | i2sodd | handle.init.mclk_output);
    }

    // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits
    // And configure the I2S with the InitStruct values
    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32;
        let mut i2scfgr = ptr::read_volatile(i2scfgr_ptr);

        // Clear all configuration bits
        i2scfgr &= !(SPI_I2SCFGR_CHLEN
            | SPI_I2SCFGR_DATLEN
            | SPI_I2SCFGR_CKPOL
            | SPI_I2SCFGR_I2SSTD
            | SPI_I2SCFGR_PCMSYNC
            | SPI_I2SCFGR_I2SCFG
            | SPI_I2SCFGR_I2SE
            | SPI_I2SCFGR_I2SMOD);

        // Set new configuration
        i2scfgr |= SPI_I2SCFGR_I2SMOD
            | handle.init.mode
            | handle.init.standard
            | handle.init.data_format
            | handle.init.cpol;

        ptr::write_volatile(i2scfgr_ptr, i2scfgr);
    }

    // Configure the I2S extended if the full duplex mode is enabled
    if handle.init.full_duplex_mode == I2S_FULLDUPLEXMODE_ENABLE {
        // EXT INIT HERE

        // Determine extended instance address
        let ext_instance = if handle.instance == 0x40003C00 {
            0x40004000 // I2S3ext
        } else if handle.instance == 0x40003800 {
            0x40003400 // I2S2ext
        } else {
            handle.error_code = HAL_I2S_ERROR_PRESCALER;
            return HalStatus::Error;
        };

        // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits for extended instance
        unsafe {
            let ext_i2scfgr_ptr = (ext_instance + 0x1C) as *mut u32;
            let mut ext_i2scfgr = ptr::read_volatile(ext_i2scfgr_ptr);
            ext_i2scfgr &= !(SPI_I2SCFGR_CHLEN
                | SPI_I2SCFGR_DATLEN
                | SPI_I2SCFGR_CKPOL
                | SPI_I2SCFGR_I2SSTD
                | SPI_I2SCFGR_PCMSYNC
                | SPI_I2SCFGR_I2SCFG
                | SPI_I2SCFGR_I2SE
                | SPI_I2SCFGR_I2SMOD);
            ptr::write_volatile(ext_i2scfgr_ptr, ext_i2scfgr);

            // Reset extended I2SPR register
            let ext_i2spr_ptr = (ext_instance + 0x20) as *mut u32;
            ptr::write_volatile(ext_i2spr_ptr, 2);
        }

        // Get the mode to be configured for the extended I2S
        if (handle.init.mode == I2S_MODE_MASTER_TX) || (handle.init.mode == I2S_MODE_SLAVE_TX) {
            tmp = I2S_MODE_SLAVE_RX;
        } else {
            // I2S_MODE_MASTER_RX || I2S_MODE_SLAVE_RX
            tmp = I2S_MODE_SLAVE_TX;
        }

        // Configure the I2S Slave with the I2S Master parameter values
        unsafe {
            let ext_i2scfgr_ptr = (ext_instance + 0x1C) as *mut u32;
            let mut ext_i2scfgr = ptr::read_volatile(ext_i2scfgr_ptr);

            ext_i2scfgr |= SPI_I2SCFGR_I2SMOD
                | tmp
                | handle.init.standard
                | handle.init.data_format
                | handle.init.cpol;

            ptr::write_volatile(ext_i2scfgr_ptr, ext_i2scfgr);
        }
    }

    handle.error_code = HAL_I2S_ERROR_NONE;
    handle.state = HalI2sState::Ready;

    HalStatus::Ok
}

pub fn hal_i2s_enable(handle: &I2sHandle) {
    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg |= SPI_I2SCFGR_I2SE; // Set I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

pub fn hal_i2s_disable(handle: &I2sHandle) {
    unsafe {
        let i2scfgr_ptr = (handle.instance + 0x1C) as *mut u32; // I2SCFGR offset
        let mut reg = ptr::read_volatile(i2scfgr_ptr);
        reg &= !SPI_I2SCFGR_I2SE; // Clear I2SE bit
        ptr::write_volatile(i2scfgr_ptr, reg);
    }
}

// Internal helper functions

fn get_i2s_clock_freq(_clock_source: u32) -> u32 {
    // Simplified: return a typical PLLI2S frequency
    50000000 // Example PLLI2S frequency
}

// Parameter validation functions

fn is_i2s_mode(mode: u32) -> bool {
    matches!(
        mode,
        I2S_MODE_SLAVE_TX | I2S_MODE_SLAVE_RX | I2S_MODE_MASTER_TX | I2S_MODE_MASTER_RX
    )
}

fn is_i2s_standard(standard: u32) -> bool {
    matches!(
        standard,
        I2S_STANDARD_PHILIPS
            | I2S_STANDARD_MSB
            | I2S_STANDARD_LSB
            | I2S_STANDARD_PCM_SHORT
            | I2S_STANDARD_PCM_LONG
    )
}

fn is_i2s_data_format(format: u32) -> bool {
    matches!(
        format,
        I2S_DATAFORMAT_16B | I2S_DATAFORMAT_16B_EXTENDED | I2S_DATAFORMAT_24B | I2S_DATAFORMAT_32B
    )
}

fn is_i2s_mclk_output(mclk: u32) -> bool {
    matches!(mclk, I2S_MCLKOUTPUT_ENABLE | I2S_MCLKOUTPUT_DISABLE)
}

fn is_i2s_audio_freq(freq: u32) -> bool {
    matches!(
        freq,
        I2S_AUDIOFREQ_192K
            | I2S_AUDIOFREQ_96K
            | I2S_AUDIOFREQ_48K
            | I2S_AUDIOFREQ_44K
            | I2S_AUDIOFREQ_32K
            | I2S_AUDIOFREQ_22K
            | I2S_AUDIOFREQ_16K
            | I2S_AUDIOFREQ_11K
            | I2S_AUDIOFREQ_8K
            | I2S_AUDIOFREQ_DEFAULT
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

// Additional constants needed
pub const HAL_MAX_DELAY: u32 = 0xFFFFFFFF;
// SPI I2S Configuration Register (I2SCFGR) bit definitions
pub const SPI_I2SCFGR_CHLEN: u32 = 0x00000001; // Channel length
pub const SPI_I2SCFGR_DATLEN: u32 = 0x00000006; // Data length mask
pub const SPI_I2SCFGR_DATLEN_0: u32 = 0x00000002; // Data length bit 0
pub const SPI_I2SCFGR_DATLEN_1: u32 = 0x00000004; // Data length bit 1
pub const SPI_I2SCFGR_CKPOL: u32 = 0x00000008; // Clock polarity
pub const SPI_I2SCFGR_I2SSTD: u32 = 0x00000030; // I2S standard selection mask
pub const SPI_I2SCFGR_I2SSTD_0: u32 = 0x00000010; // I2S standard bit 0
pub const SPI_I2SCFGR_I2SSTD_1: u32 = 0x00000020; // I2S standard bit 1
pub const SPI_I2SCFGR_PCMSYNC: u32 = 0x00000080; // PCM frame synchronization
pub const SPI_I2SCFGR_I2SCFG: u32 = 0x00000300; // I2S configuration mode mask
pub const SPI_I2SCFGR_I2SCFG_0: u32 = 0x00000100; // I2S configuration mode bit 0
pub const SPI_I2SCFGR_I2SCFG_1: u32 = 0x00000200; // I2S configuration mode bit 1
pub const SPI_I2SCFGR_I2SE: u32 = 0x00000400; // I2S Enable
pub const SPI_I2SCFGR_I2SMOD: u32 = 0x00000800; // I2S mode selection

// Combined masks for clearing multiple bits
pub const SPI_I2SCFGR_CLEAR_MASK: u32 = SPI_I2SCFGR_CHLEN
    | SPI_I2SCFGR_DATLEN
    | SPI_I2SCFGR_CKPOL
    | SPI_I2SCFGR_I2SSTD
    | SPI_I2SCFGR_PCMSYNC
    | SPI_I2SCFGR_I2SCFG
    | SPI_I2SCFGR_I2SE
    | SPI_I2SCFGR_I2SMOD;

// Helper functions for flag checking
fn i2s_get_flag_status(hi2s: &I2sHandle, flag: u32) -> bool {
    let sr = unsafe { ptr::read_volatile((hi2s.instance + 0x08) as *const u32) };
    (sr & flag) != 0
}

// SysTick register addresses for STM32F4
const SYST_CSR: u32 = 0xE000E010; // SysTick Control and Status Register
const SYST_RVR: u32 = 0xE000E014; // SysTick Reload Value Register
const SYST_CVR: u32 = 0xE000E018; // SysTick Current Value Register

// Global tick counter (would typically be in BSS section)
static mut UWTICK: u32 = 0;

pub fn hal_init_tick(hclk_frequency: u32) {
    // Configure SysTick to generate interrupt every 1ms
    let reload_value = (hclk_frequency / 1000) - 1;

    unsafe {
        // Set reload value
        ptr::write_volatile(SYST_RVR as *mut u32, reload_value);

        // Clear current value
        ptr::write_volatile(SYST_CVR as *mut u32, 0);

        // Enable SysTick with processor clock and interrupt
        ptr::write_volatile(SYST_CSR as *mut u32, 0x7); // CLKSOURCE | TICKINT | ENABLE
    }
}

fn hal_get_tick() -> u32 {
    unsafe { UWTICK }
}

pub fn hal_inc_tick() {
    unsafe {
        UWTICK = UWTICK.wrapping_add(1);
    }
}

fn i2s_wait_flag_state_until_timeout(
    hi2s: &mut I2sHandle,
    flag: u32,
    state: bool,
    timeout: u32,
) -> HalStatus {
    let tick_start = hal_get_tick();

    let mut curr_state = i2s_get_flag_status(hi2s, flag);
    while curr_state != state {
        let elapsed = hal_get_tick();
        let elapsed = elapsed.wrapping_sub(tick_start);

        if timeout != HAL_MAX_DELAY && elapsed > timeout {
            hi2s.state = HalI2sState::Ready;
            hi2s.lock = HalLock::Unlocked;

            return HalStatus::Timeout;
        }

        curr_state = i2s_get_flag_status(hi2s, flag);
    }

    HalStatus::Ok
}

fn i2s_wait_flag_state_until_timeout_instance(
    instance: u32,
    flag: u32,
    state: bool,
    timeout: u32,
) -> HalStatus {
    let tick_start = hal_get_tick();

    let mut curr_state = i2s_get_flag_status_instance(instance, flag);
    while curr_state != state {
        let elapsed = hal_get_tick();
        let elapsed = elapsed.wrapping_sub(tick_start);

        if timeout != HAL_MAX_DELAY && elapsed > timeout {
            return HalStatus::Timeout;
        }

        curr_state = i2s_get_flag_status_instance(instance, flag);
    }

    HalStatus::Ok
}

fn i2s_get_flag_status_instance(instance: u32, i2s_flag: u32) -> bool {
    unsafe {
        let sr_ptr = (instance + 0x08) as *const u32; // SR offset
        let sr = ptr::read_volatile(sr_ptr);
        (sr & i2s_flag) != 0
    }
}

pub fn hal_i2s_transmit(hi2s: &mut I2sHandle, p_data: &[u16], timeout: u32) -> HalStatus {
    if hi2s.state != HalI2sState::Ready {
        return HalStatus::Busy;
    }

    if p_data.is_empty() {
        return HalStatus::Error;
    }

    // Set state to busy transmission
    hi2s.state = HalI2sState::BusyTx;
    hi2s.error_code = HAL_I2S_ERROR_NONE;

    let size = p_data.len();
    let mut tx_data_ptr = p_data.as_ptr();
    let mut tmp_size = size;

    // Check if the I2S is already enabled
    let i2scfgr = unsafe { ptr::read_volatile((hi2s.instance + 0x1C) as *const u32) };
    if (i2scfgr & SPI_I2SCFGR_I2SE) == 0 {
        // Enable I2S peripheral
        unsafe {
            ptr::write_volatile(
                (hi2s.instance + 0x1C) as *mut u32,
                i2scfgr | SPI_I2SCFGR_I2SE,
            );
        }
    }

    // Start the transfer
    while tmp_size > 0 {
        // Wait until TXE flag is set
        if i2s_wait_flag_state_until_timeout(hi2s, I2S_FLAG_TXE, true, timeout) != HalStatus::Ok {
            // Set the error code and state are already set by the timeout function
            return HalStatus::Timeout;
        }

        // Write data to DR register
        let data = unsafe { ptr::read(tx_data_ptr) };
        unsafe {
            ptr::write_volatile((hi2s.instance + 0x0C) as *mut u16, data);
        }

        tx_data_ptr = unsafe { tx_data_ptr.add(1) };
        tmp_size -= 1;
    }

    // Wait until Busy flag is reset
    if i2s_wait_flag_state_until_timeout(hi2s, I2S_FLAG_BSY, false, timeout) != HalStatus::Ok {
        // Set the error code
        hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
        hi2s.state = HalI2sState::Ready;
        return HalStatus::Timeout;
    }

    hi2s.state = HalI2sState::Ready;
    HalStatus::Ok
}

pub fn hal_i2sex_transmit_receive(
    hi2s: &mut I2sHandle,
    p_tx_data: &[u16],
    p_rx_data: &mut [u16],
    timeout: u32,
) -> HalStatus {
    let size = p_tx_data.len().min(p_rx_data.len());

    if hi2s.state != HalI2sState::Ready {
        return HalStatus::Busy;
    }

    if p_tx_data.is_empty() || p_rx_data.is_empty() || size == 0 {
        defmt::trace!("exit 1");
        return HalStatus::Error;
    }

    // Process Locked
    hi2s.lock = HalLock::Locked;

    // Check the data format to determine transfer size
    let i2scfgr = unsafe { ptr::read_volatile((hi2s.instance + 0x1C) as *const u32) };
    let tmp1 = i2scfgr & (SPI_I2SCFGR_DATLEN | SPI_I2SCFGR_CHLEN);

    let (tx_xfer_size, rx_xfer_size) =
        if (tmp1 == I2S_DATAFORMAT_24B) || (tmp1 == I2S_DATAFORMAT_32B) {
            (size << 1, size << 1) // Double the size for 24/32-bit formats
        } else {
            (size, size) // Normal size for 16-bit formats
        };

    let mut tx_xfer_count = tx_xfer_size;
    let mut rx_xfer_count = rx_xfer_size;
    let mut tx_data_ptr = p_tx_data.as_ptr();
    let mut rx_data_ptr = p_rx_data.as_mut_ptr();

    // Set state and reset error code
    hi2s.error_code = HAL_I2S_ERROR_NONE;
    hi2s.state = HalI2sState::BusyTxRx;

    // Get the I2S mode configuration
    let i2s_mode = i2scfgr & SPI_I2SCFGR_I2SCFG;

    // Determine extended instance address
    let ext_instance = if hi2s.instance == 0x40003C00 {
        0x40004000 // I2S3ext
    } else if hi2s.instance == 0x40003800 {
        0x40003400 // I2S2ext
    } else {
        defmt::trace!("exit 2");
        hi2s.state = HalI2sState::Ready;
        hi2s.lock = HalLock::Unlocked;
        return HalStatus::Error;
    };

    // Check if the I2S_MODE_MASTER_TX or I2S_MODE_SLAVE_TX Mode is selected
    if (i2s_mode == I2S_MODE_MASTER_TX) || (i2s_mode == I2S_MODE_SLAVE_TX) {
        // Prepare the First Data before enabling the I2S
        unsafe {
            ptr::write_volatile((hi2s.instance + 0x0C) as *mut u16, *tx_data_ptr);
        }
        tx_data_ptr = unsafe { tx_data_ptr.add(1) };
        tx_xfer_count -= 1;

        // Enable I2Sext(receiver) before enabling I2Sx peripheral
        unsafe {
            let i2scfgr_ext_ptr = (ext_instance + 0x1C) as *mut u32;
            let i2scfgr_ext = ptr::read_volatile(i2scfgr_ext_ptr);
            ptr::write_volatile(i2scfgr_ext_ptr, i2scfgr_ext | SPI_I2SCFGR_I2SE);
        }

        // Enable I2Sx peripheral
        unsafe {
            ptr::write_volatile(
                (hi2s.instance + 0x1C) as *mut u32,
                i2scfgr | SPI_I2SCFGR_I2SE,
            );
        }

        // Clear the Overrun Flag if in master TX mode
        if i2s_mode == I2S_MODE_MASTER_TX {
            // Clear overrun flag by reading DR then SR of extended instance
            unsafe {
                let _dummy = ptr::read_volatile((ext_instance + 0x0C) as *const u32);
                let _dummy = ptr::read_volatile((ext_instance + 0x08) as *const u32);
            }
        }

        // Main transfer loop
        while (rx_xfer_count > 0) || (tx_xfer_count > 0) {
            // Transmit data if available
            if tx_xfer_count > 0 {
                // Wait until TXE flag is set on main instance
                if i2s_wait_flag_state_until_timeout_instance(
                    hi2s.instance,
                    I2S_FLAG_TXE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
                    defmt::trace!("exit 3");
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Write Data on DR register of main instance
                unsafe {
                    ptr::write_volatile((hi2s.instance + 0x0C) as *mut u16, *tx_data_ptr);
                }
                tx_data_ptr = unsafe { tx_data_ptr.add(1) };
                tx_xfer_count -= 1;

                // Check if an underrun occurs (only for slave TX mode)
                if i2s_mode == I2S_MODE_SLAVE_TX {
                    let sr = unsafe { ptr::read_volatile((hi2s.instance + 0x08) as *const u32) };
                    if (sr & I2S_FLAG_UDR) != 0 {
                        // Clear Underrun flag
                        unsafe {
                            let _dummy = ptr::read_volatile((hi2s.instance + 0x08) as *const u32);
                        }
                        hi2s.error_code |= HAL_I2S_ERROR_UDR;
                    }
                }
            }

            // Receive data if available
            if rx_xfer_count > 0 {
                // Wait until RXNE flag is set on extended instance
                if i2s_wait_flag_state_until_timeout_instance(
                    ext_instance,
                    I2S_FLAG_RXNE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
                    defmt::trace!("exit 4");
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Read Data from DR register of extended instance
                let rx_data = unsafe { ptr::read_volatile((ext_instance + 0x0C) as *const u16) };
                unsafe {
                    ptr::write(rx_data_ptr, rx_data);
                }
                rx_data_ptr = unsafe { rx_data_ptr.add(1) };
                rx_xfer_count -= 1;

                // Check if an overrun occurs on extended instance
                let sr_ext = unsafe { ptr::read_volatile((ext_instance + 0x08) as *const u32) };
                if (sr_ext & I2S_FLAG_OVR) != 0 {
                    // Clear Overrun flag
                    unsafe {
                        let _dummy = ptr::read_volatile((ext_instance + 0x0C) as *const u32);
                        let _dummy = ptr::read_volatile((ext_instance + 0x08) as *const u32);
                    }
                    hi2s.error_code |= HAL_I2S_ERROR_OVR;
                }
            }
        }
    } else {
        // The I2S_MODE_MASTER_RX or I2S_MODE_SLAVE_RX Mode is selected

        // Prepare the First Data before enabling the I2S (write to extended instance)
        unsafe {
            ptr::write_volatile((ext_instance + 0x0C) as *mut u16, *tx_data_ptr);
        }
        tx_data_ptr = unsafe { tx_data_ptr.add(1) };
        tx_xfer_count -= 1;

        // Enable I2Sext(transmitter) after enabling I2Sx peripheral
        unsafe {
            let i2scfgr_ext_ptr = (ext_instance + 0x1C) as *mut u32;
            let i2scfgr_ext = ptr::read_volatile(i2scfgr_ext_ptr);
            ptr::write_volatile(i2scfgr_ext_ptr, i2scfgr_ext | SPI_I2SCFGR_I2SE);
        }

        // Enable I2S peripheral before the I2Sext
        unsafe {
            ptr::write_volatile(
                (hi2s.instance + 0x1C) as *mut u32,
                i2scfgr | SPI_I2SCFGR_I2SE,
            );
        }

        // Clear the Overrun Flag if in master RX mode
        if i2s_mode == I2S_MODE_MASTER_RX {
            // Clear overrun flag by reading DR then SR of main instance
            unsafe {
                let _dummy = ptr::read_volatile((hi2s.instance + 0x0C) as *const u32);
                let _dummy = ptr::read_volatile((hi2s.instance + 0x08) as *const u32);
            }
        }

        // Main transfer loop
        while (rx_xfer_count > 0) || (tx_xfer_count > 0) {
            // Transmit data if available (use extended instance)
            if tx_xfer_count > 0 {
                // Wait until TXE flag is set on extended instance
                if i2s_wait_flag_state_until_timeout_instance(
                    ext_instance,
                    I2S_FLAG_TXE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
                    defmt::trace!("exit 5");
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Write Data on DR register of extended instance
                unsafe {
                    ptr::write_volatile((ext_instance + 0x0C) as *mut u16, *tx_data_ptr);
                }
                tx_data_ptr = unsafe { tx_data_ptr.add(1) };
                tx_xfer_count -= 1;

                // Check if an underrun occurs on extended instance (only for slave RX mode)
                if i2s_mode == I2S_MODE_SLAVE_RX {
                    let sr_ext = unsafe { ptr::read_volatile((ext_instance + 0x08) as *const u32) };
                    if (sr_ext & I2S_FLAG_UDR) != 0 {
                        // Clear Underrun flag
                        unsafe {
                            let _dummy = ptr::read_volatile((ext_instance + 0x08) as *const u32);
                        }
                        hi2s.error_code |= HAL_I2S_ERROR_UDR;
                    }
                }
            }

            // Receive data if available (use main instance)
            if rx_xfer_count > 0 {
                // Wait until RXNE flag is set on main instance
                if i2s_wait_flag_state_until_timeout_instance(
                    hi2s.instance,
                    I2S_FLAG_RXNE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
                    defmt::trace!("exit 6");
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Read Data from DR register of main instance
                let rx_data = unsafe { ptr::read_volatile((hi2s.instance + 0x0C) as *const u16) };
                unsafe {
                    ptr::write(rx_data_ptr, rx_data);
                }
                rx_data_ptr = unsafe { rx_data_ptr.add(1) };
                rx_xfer_count -= 1;

                // Check if an overrun occurs on main instance
                let sr = unsafe { ptr::read_volatile((hi2s.instance + 0x08) as *const u32) };
                if (sr & I2S_FLAG_OVR) != 0 {
                    // Clear Overrun flag
                    unsafe {
                        let _dummy = ptr::read_volatile((hi2s.instance + 0x0C) as *const u32);
                        let _dummy = ptr::read_volatile((hi2s.instance + 0x08) as *const u32);
                    }
                    hi2s.error_code |= HAL_I2S_ERROR_OVR;
                }
            }
        }
    }

    // Process Unlocked
    hi2s.lock = HalLock::Unlocked;
    hi2s.state = HalI2sState::Ready;

    HalStatus::Ok
}

pub fn hal_i2s_msp_init() {
    use embassy_stm32::pac::{
        gpio::vals::{Moder, Ospeedr, Ot, Pupdr},
        GPIOA, GPIOB, GPIOC, RCC,
    };

    // Enable peripheral clocks
    RCC.apb1enr().modify(|w| w.set_spi3en(true));
    RCC.ahb1enr().modify(|w| {
        w.set_gpioaen(true);
        w.set_gpioben(true);
        w.set_gpiocen(true);
    });

    // Configure PA15 -> I2S3_WS (AF6, Push-Pull, No Pull, Low Speed)
    GPIOA.moder().modify(|w| w.set_moder(15, Moder::ALTERNATE));
    GPIOA.otyper().modify(|w| w.set_ot(15, Ot::PUSH_PULL));
    GPIOA.pupdr().modify(|w| w.set_pupdr(15, Pupdr::FLOATING));
    GPIOA
        .ospeedr()
        .modify(|w| w.set_ospeedr(15, Ospeedr::LOW_SPEED));
    GPIOA.afr(1).modify(|w| w.set_afr(15 - 8, 6)); // AF6

    // Configure PC10 -> I2S3_CK (AF6, Push-Pull, No Pull, Low Speed)
    GPIOC.moder().modify(|w| w.set_moder(10, Moder::ALTERNATE));
    GPIOC.otyper().modify(|w| w.set_ot(10, Ot::PUSH_PULL));
    GPIOC.pupdr().modify(|w| w.set_pupdr(10, Pupdr::FLOATING));
    GPIOC
        .ospeedr()
        .modify(|w| w.set_ospeedr(10, Ospeedr::LOW_SPEED));
    GPIOC.afr(1).modify(|w| w.set_afr(10 - 8, 6)); // AF6

    // Configure PB4 -> I2S3_ext_SD (AF7, Push-Pull, No Pull, Low Speed)
    GPIOB.moder().modify(|w| w.set_moder(4, Moder::ALTERNATE));
    GPIOB.otyper().modify(|w| w.set_ot(4, Ot::PUSH_PULL));
    GPIOB.pupdr().modify(|w| w.set_pupdr(4, Pupdr::FLOATING));
    GPIOB
        .ospeedr()
        .modify(|w| w.set_ospeedr(4, Ospeedr::LOW_SPEED));
    GPIOB.afr(0).modify(|w| w.set_afr(4, 7)); // AF7 for I2S3ext

    // Configure PB5 -> I2S3_SD (AF6, Push-Pull, No Pull, Low Speed)
    GPIOB.moder().modify(|w| w.set_moder(5, Moder::ALTERNATE));
    GPIOB.otyper().modify(|w| w.set_ot(5, Ot::PUSH_PULL));
    GPIOB.pupdr().modify(|w| w.set_pupdr(5, Pupdr::FLOATING));
    GPIOB
        .ospeedr()
        .modify(|w| w.set_ospeedr(5, Ospeedr::LOW_SPEED));
    GPIOB.afr(0).modify(|w| w.set_afr(5, 6)); // AF6

    // Note: DMA initialization skipped as requested
    // Note: I2S3 interrupt configuration skipped (HAL_NVIC_SetPriority/EnableIRQ)
}
