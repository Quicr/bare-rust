use core::ptr;
use defmt::Format;
use embassy_stm32::pac::spi::{vals::*, Spi};
use num_enum::IntoPrimitive;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum Mode {
    SlaveTx = 0x00000000,
    SlaveRx = 0x00000100,
    MasterTx = 0x00000200,
    MasterRx = 0x00000300,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum Standard {
    Philips = 0x00000000,
    Msb = 0x00000010,
    Lsb = 0x00000020,
    PcmShort = 0x00000030,
    PcmLong = 0x000000B0,
}

// XXX These get converted to DATLEN values, but they look like they don't correspond to the
// metapac values: https://docs.embassy.dev/stm32-metapac/git/stm32f405rg/spi/vals/enum.Datlen.html
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum DataFormat {
    Data16b = 0x00000000,
    Data16bExtended = 0x00000001,
    Data24b = 0x00000003,
    Data32b = 0x00000005,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum MclkOutput {
    Disable = 0x00000000,
    Enable = 0x00000200,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum AudioFreq {
    Hz192k = 192000,
    Hz96k = 96000,
    Hz48k = 48000,
    Hz44k = 44100,
    Hz32k = 32000,
    Hz22k = 22050,
    Hz16k = 16000,
    Hz11k = 11025,
    Hz8k = 8000,
    Default = 2,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum Cpol {
    Low = 0x00000000,
    High = 0x00000008,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum ClockSource {
    Plli2s = 0x00000000,
    Ext = 0x00000001,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
pub enum FullDuplexMode {
    Disable = 0x00000000,
    Enable = 0x00000001,
}

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
    pub mode: Mode,
    pub standard: Standard,
    pub data_format: DataFormat,
    pub mclk_output: MclkOutput,
    pub audio_freq: AudioFreq,
    pub cpol: Cpol,
    pub clock_source: ClockSource,
    pub full_duplex_mode: FullDuplexMode,
}

impl Default for I2sInit {
    fn default() -> Self {
        Self {
            mode: Mode::SlaveTx,
            standard: Standard::Philips,
            data_format: DataFormat::Data16b,
            mclk_output: MclkOutput::Disable,
            audio_freq: AudioFreq::Default,
            cpol: Cpol::Low,
            clock_source: ClockSource::Plli2s,
            full_duplex_mode: FullDuplexMode::Disable,
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
    pub regs: Spi,
    pub regs_ext: Spi,
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
            regs: unsafe { Spi::from_ptr(instance as *mut ()) },
            regs_ext: unsafe { Spi::from_ptr(i2s_ext_instance(instance) as *mut ()) },
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
// XXX(RLB) These have been translated to use the register blocks, but not actually tested.

pub fn i2s_ext_instance(instance: u32) -> u32 {
    if instance == SPI2_BASE {
        I2S2EXT_BASE
    } else {
        I2S3EXT_BASE
    }
}

pub fn hal_i2s_ext_enable(handle: &I2sHandle) {
    handle.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));
}

pub fn hal_i2s_ext_disable(handle: &I2sHandle) {
    handle.regs_ext.i2scfgr().modify(|w| w.set_i2se(false));
}

pub fn hal_i2s_ext_clear_ovr_flag(handle: &I2sHandle) {
    let _ = handle.regs_ext.dr().read();
    let _ = handle.regs_ext.sr().read();
}

pub fn hal_i2s_init(handle: &mut I2sHandle) -> HalStatus {
    let i2sdiv: u32;
    let mut i2sodd: u32;
    let mut packetlength: u32;
    let mut tmp: u32;
    let i2sclk: u32;

    if handle.state == HalI2sState::Reset {
        // Allocate lock resource and initialize it
        handle.lock = HalLock::Unlocked;

        // Init the low level hardware: GPIO, CLOCK, NVIC... (already done by hal_i2s_msp_init)
    }

    handle.state = HalI2sState::Busy;

    // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits
    handle.regs.i2scfgr().modify(|w| {
        // TODO use semantic modifications
        w.0 &= !(SPI_I2SCFGR_CHLEN
            | SPI_I2SCFGR_DATLEN
            | SPI_I2SCFGR_CKPOL
            | SPI_I2SCFGR_I2SSTD
            | SPI_I2SCFGR_PCMSYNC
            | SPI_I2SCFGR_I2SCFG
            | SPI_I2SCFGR_I2SE
            | SPI_I2SCFGR_I2SMOD);
    });

    handle.regs.i2spr().modify(|w| w.0 = 0x0002);

    // I2SPR: I2SDIV and ODD Calculation
    // If the requested audio frequency is not the default, compute the prescaler
    if handle.init.audio_freq != AudioFreq::Default {
        // Check the frame length (For the Prescaler computing)
        if handle.init.data_format == DataFormat::Data16b {
            // Packet length is 16 bits
            packetlength = 16;
        } else {
            // Packet length is 32 bits
            packetlength = 32;
        }

        // I2S standard
        if matches!(
            handle.init.standard,
            Standard::Philips | Standard::Msb | Standard::Lsb
        ) {
            // In I2S standard packet length is multiplied by 2
            packetlength = packetlength * 2;
        }

        // Get the source clock value (simplified - use PLLI2S)
        i2sclk = get_i2s_clock_freq(handle.init.clock_source.into());

        // Compute the Real divider depending on the MCLK output state, with a floating point
        if handle.init.mclk_output == MclkOutput::Enable {
            // MCLK output is enabled
            let audio_freq: u32 = handle.init.audio_freq.into();
            tmp = (((i2sclk / 256) * 10) / audio_freq) + 5;
        } else {
            // MCLK output is disabled
            let audio_freq: u32 = handle.init.audio_freq.into();
            tmp = (((i2sclk / packetlength) * 10) / audio_freq) + 5;
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
    let mclk_output: u32 = handle.init.mclk_output.into();
    handle.regs.i2spr().modify(|w| {
        // TODO use semantic modifiers
        w.0 = i2sdiv | i2sodd | mclk_output;
    });

    // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits
    // And configure the I2S with the InitStruct values
    handle.regs.i2scfgr().modify(|w| {
        // TODO use semantic modifiers
        let mode: u32 = handle.init.mode.into();
        let standard: u32 = handle.init.standard.into();
        let data_format: u32 = handle.init.data_format.into();
        let cpol: u32 = handle.init.cpol.into();
        w.0 = SPI_I2SCFGR_I2SMOD | mode | standard | data_format | cpol;
    });

    // Configure the I2S extended if the full duplex mode is enabled
    if handle.init.full_duplex_mode == FullDuplexMode::Enable {
        // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits for extended instance
        handle.regs_ext.i2scfgr().modify(|w| {
            // TODO use semantic modifications
            w.0 &= !(SPI_I2SCFGR_CHLEN
                | SPI_I2SCFGR_DATLEN
                | SPI_I2SCFGR_CKPOL
                | SPI_I2SCFGR_I2SSTD
                | SPI_I2SCFGR_PCMSYNC
                | SPI_I2SCFGR_I2SCFG
                | SPI_I2SCFGR_I2SE
                | SPI_I2SCFGR_I2SMOD);
        });

        // Get the mode to be configured for the extended I2S
        let ext_mode = match handle.init.mode {
            Mode::MasterTx | Mode::SlaveTx => Mode::SlaveRx,
            Mode::MasterRx | Mode::SlaveRx => Mode::SlaveTx,
        };

        // Configure the I2S Slave with the I2S Master parameter values
        handle.regs_ext.i2scfgr().modify(|w| {
            // TODO use semantic modifiers
            let mode: u32 = ext_mode.into();
            let standard: u32 = handle.init.standard.into();
            let data_format: u32 = handle.init.data_format.into();
            let cpol: u32 = handle.init.cpol.into();
            w.0 = SPI_I2SCFGR_I2SMOD | mode | standard | data_format | cpol;
        });
    }

    handle.error_code = HAL_I2S_ERROR_NONE;
    handle.state = HalI2sState::Ready;

    HalStatus::Ok
}

pub fn hal_i2s_enable(handle: &I2sHandle) {
    handle.regs.i2scfgr().modify(|w| w.set_i2se(true));
}

pub fn hal_i2s_disable(handle: &I2sHandle) {
    handle.regs.i2scfgr().modify(|w| w.set_i2se(false));
}

// Internal helper functions

fn get_i2s_clock_freq(_clock_source: u32) -> u32 {
    // Simplified: return a typical PLLI2S frequency
    50000000 // Example PLLI2S frequency
}

// Parameter validation functions

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
// TODO Replace this with actual flag reads
fn i2s_get_flag_status(hi2s: &I2sHandle, flag: u32) -> bool {
    let sr = hi2s.regs.sr().read();
    (sr.0 & flag) != 0
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
    regs: &Spi,
    flag: u32,
    state: bool,
    timeout: u32,
) -> HalStatus {
    let tick_start = hal_get_tick();

    let mut curr_state = i2s_get_flag_status_instance(regs, flag);
    while curr_state != state {
        let elapsed = hal_get_tick();
        let elapsed = elapsed.wrapping_sub(tick_start);

        if timeout != HAL_MAX_DELAY && elapsed > timeout {
            return HalStatus::Timeout;
        }

        curr_state = i2s_get_flag_status_instance(regs, flag);
    }

    HalStatus::Ok
}

fn i2s_get_flag_status_instance(regs: &Spi, i2s_flag: u32) -> bool {
    let sr = regs.sr().read();
    (sr.0 & i2s_flag) != 0
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

    // Check if the I2S is already enabled
    hi2s.regs.i2scfgr().modify(|w| {
        if !w.i2se() {
            w.set_i2se(true);
        }
    });

    // Start the transfer
    // XXX: This ought to be a range for loop, but that seems to screw things up
    for i in 0..p_data.len() {
        // while i < size {
        // Wait until TXE flag is set
        if i2s_wait_flag_state_until_timeout(hi2s, I2S_FLAG_TXE, true, timeout) != HalStatus::Ok {
            // Set the error code and state are already set by the timeout function
            return HalStatus::Timeout;
        }

        // Write data to DR register
        hi2s.regs.dr().write(|w| w.set_dr(p_data[i]));
    }

    // Wait until Busy flag is reset
    // XXX In the C code, this is only done when in SLAVE_TX or SLAVE_RX mode
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
        return HalStatus::Error;
    }

    // Process Locked
    hi2s.lock = HalLock::Locked;

    // Check the data format to determine transfer size
    // TODO make this more elegant, e.g., with enum mapping
    let i2scfgr = hi2s.regs.i2scfgr().read();
    let datlen = i2scfgr.datlen().to_bits();
    let chlen = i2scfgr.chlen().to_bits();
    let data_format = ((datlen << 1) | chlen) as u32;
    let (tx_xfer_size, rx_xfer_size) = if (data_format == DataFormat::Data24b as u32)
        || (data_format == DataFormat::Data32b as u32)
    {
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
    let i2s_mode = match i2scfgr.i2scfg() {
        I2scfg::SLAVE_TX => Mode::SlaveTx,
        I2scfg::SLAVE_RX => Mode::SlaveRx,
        I2scfg::MASTER_TX => Mode::MasterTx,
        I2scfg::MASTER_RX => Mode::MasterRx,
    };

    // Determine extended instance address
    let ext_instance = if hi2s.instance == 0x40003C00 {
        0x40004000 // I2S3ext
    } else if hi2s.instance == 0x40003800 {
        0x40003400 // I2S2ext
    } else {
        hi2s.state = HalI2sState::Ready;
        hi2s.lock = HalLock::Unlocked;
        return HalStatus::Error;
    };

    // Check if the I2S_MODE_MASTER_TX or I2S_MODE_SLAVE_TX Mode is selected
    if (i2s_mode == Mode::MasterTx) || (i2s_mode == Mode::SlaveTx) {
        // Prepare the First Data before enabling the I2S
        unsafe {
            ptr::write_volatile((hi2s.instance + 0x0C) as *mut u16, *tx_data_ptr);
        }
        tx_data_ptr = unsafe { tx_data_ptr.add(1) };
        tx_xfer_count -= 1;

        // Enable peripherals
        hi2s.regs.i2scfgr().modify(|w| w.set_i2se(true));
        hi2s.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));

        // Clear the Overrun Flag if in master TX mode
        if i2s_mode == Mode::MasterTx {
            // Clear overrun flag by reading DR then SR of extended instance
            let _ = hi2s.regs_ext.dr().read();
            let _ = hi2s.regs_ext.sr().read();
        }

        // Main transfer loop
        while (rx_xfer_count > 0) || (tx_xfer_count > 0) {
            // Transmit data if available
            if tx_xfer_count > 0 {
                // Wait until TXE flag is set on main instance
                if i2s_wait_flag_state_until_timeout_instance(
                    &hi2s.regs,
                    I2S_FLAG_TXE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
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
                if i2s_mode == Mode::SlaveTx {
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
                    &hi2s.regs_ext,
                    I2S_FLAG_RXNE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
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
                // XXX Things break if this read isn't here
                let _sr_ext = unsafe { ptr::read_volatile((ext_instance + 0x08) as *const u32) };
                if hi2s.regs_ext.sr().read().ovr() {
                    // Clear Overrun flag
                    let _ = hi2s.regs_ext.dr().read();
                    let _ = hi2s.regs_ext.sr().read();
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

        // Enable the peripherals
        hi2s.regs.i2scfgr().modify(|w| w.set_i2se(true));
        hi2s.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));

        // Clear the Overrun Flag if in master RX mode
        if i2s_mode == Mode::MasterRx {
            // Clear overrun flag by reading DR then SR of main instance
            let _ = hi2s.regs.dr().read();
            let _ = hi2s.regs.sr().read();
        }

        // Main transfer loop
        while (rx_xfer_count > 0) || (tx_xfer_count > 0) {
            // Transmit data if available (use extended instance)
            if tx_xfer_count > 0 {
                // Wait until TXE flag is set on extended instance
                if i2s_wait_flag_state_until_timeout_instance(
                    &hi2s.regs_ext,
                    I2S_FLAG_TXE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
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
                if i2s_mode == Mode::SlaveRx {
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
                    &hi2s.regs,
                    I2S_FLAG_RXNE,
                    true,
                    timeout,
                ) != HalStatus::Ok
                {
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
                // XXX This read should be unnecessary, but things lock up without it
                let sr = unsafe { ptr::read_volatile((hi2s.instance + 0x08) as *const u32) };
                if hi2s.regs.sr().read().ovr() {
                    // Clear Overrun flag
                    let _ = hi2s.regs.dr().read();
                    let _ = hi2s.regs.sr().read();
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
