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
pub struct Config {
    pub mode: Mode,
    pub standard: Standard,
    pub data_format: DataFormat,
    pub mclk_output: MclkOutput,
    pub audio_freq: AudioFreq,
    pub cpol: Cpol,
    pub clock_source: ClockSource,
    pub full_duplex_mode: FullDuplexMode,
}

impl Default for Config {
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
const SPI2_BASE: u32 = 0x40003800;
const SPI3_BASE: u32 = 0x40003C00;
const I2S2EXT_BASE: u32 = 0x40003400;
const I2S3EXT_BASE: u32 = 0x40004000;

const SPI2: Spi = unsafe { Spi::from_ptr(0x40003800 as *mut ()) };
const SPI3: Spi = unsafe { Spi::from_ptr(0x40003C00 as *mut ()) };
const I2S2EXT: Spi = unsafe { Spi::from_ptr(0x40003400 as *mut ()) };
const I2S3EXT: Spi = unsafe { Spi::from_ptr(0x40004000 as *mut ()) };

pub struct I2sHandle {
    regs: Spi,
    regs_ext: Spi,
    tx_buff_ptr: *mut u16,
    tx_xfer_size: u16,
    tx_xfer_count: u16,
    rx_buff_ptr: *mut u16,
    rx_xfer_size: u16,
    rx_xfer_count: u16,
    lock: HalLock,
    state: HalI2sState,
    pub error_code: u32,
}

impl I2sHandle {
    fn new(regs: Spi, regs_ext: Spi) -> Self {
        Self {
            // instance,
            regs,
            regs_ext,
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
        Self::new(SPI2, I2S2EXT)
    }

    pub fn new_spi3() -> Self {
        Self::new(SPI3, I2S3EXT)
    }
}

// Extended I2S macros (translated from C)
fn i2s_ext_instance(instance: u32) -> u32 {
    if instance == SPI2_BASE {
        I2S2EXT_BASE
    } else {
        I2S3EXT_BASE
    }
}

pub fn hal_i2s_init(handle: &mut I2sHandle, config: Config) -> HalStatus {
    let i2sdiv: u32;
    let mut i2sodd: u32;
    let mut packetlength: u32;

    if handle.state == HalI2sState::Reset {
        // Allocate lock resource and initialize it
        handle.lock = HalLock::Unlocked;

        // Init the low level hardware: GPIO, CLOCK, NVIC... (already done by hal_i2s_msp_init)
    }

    handle.state = HalI2sState::Busy;

    // I2SPR: I2SDIV and ODD Calculation
    // If the requested audio frequency is not the default, compute the prescaler
    if config.audio_freq != AudioFreq::Default {
        // Check the frame length (For the Prescaler computing)
        if config.data_format == DataFormat::Data16b {
            // Packet length is 16 bits
            packetlength = 16;
        } else {
            // Packet length is 32 bits
            packetlength = 32;
        }

        // I2S standard
        if matches!(
            config.standard,
            Standard::Philips | Standard::Msb | Standard::Lsb
        ) {
            // In I2S standard packet length is multiplied by 2
            packetlength = packetlength * 2;
        }

        // Get the source clock value (simplified - use PLLI2S)
        // XXX Our i2s clock is set to 50MHz.  We should pull this from RCC or something.
        let i2sclk = 50_000_000;

        // Compute the Real divider depending on the MCLK output state, with a floating point
        let mut tmp = if config.mclk_output == MclkOutput::Enable {
            // MCLK output is enabled
            let audio_freq: u32 = config.audio_freq.into();
            (((i2sclk / 256) * 10) / audio_freq) + 5
        } else {
            // MCLK output is disabled
            let audio_freq: u32 = config.audio_freq.into();
            (((i2sclk / packetlength) * 10) / audio_freq) + 5
        };

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
    let mclk_output: u32 = config.mclk_output.into();
    handle.regs.i2spr().modify(|w| {
        // TODO use semantic modifiers
        w.0 = i2sdiv | i2sodd | mclk_output;
    });

    // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits
    // And configure the I2S with the InitStruct values
    handle.regs.i2scfgr().modify(|w| {
        // TODO use semantic modifiers
        let mode: u32 = config.mode.into();
        let standard: u32 = config.standard.into();
        let data_format: u32 = config.data_format.into();
        let cpol: u32 = config.cpol.into();
        w.0 = mode | standard | data_format | cpol;
        w.set_i2smod(true);
    });

    // Configure the I2S extended if the full duplex mode is enabled
    if config.full_duplex_mode == FullDuplexMode::Enable {
        // Get the mode to be configured for the extended I2S
        let ext_mode = match config.mode {
            Mode::MasterTx | Mode::SlaveTx => Mode::SlaveRx,
            Mode::MasterRx | Mode::SlaveRx => Mode::SlaveTx,
        };

        // Configure the I2S Slave with the I2S Master parameter values
        handle.regs_ext.i2scfgr().modify(|w| {
            // TODO use semantic modifiers
            let mode: u32 = ext_mode.into();
            let standard: u32 = config.standard.into();
            let data_format: u32 = config.data_format.into();
            let cpol: u32 = config.cpol.into();
            w.0 = mode | standard | data_format | cpol;
            w.set_i2smod(true);
        });
    }

    handle.error_code = HAL_I2S_ERROR_NONE;
    handle.state = HalI2sState::Ready;

    HalStatus::Ok
}

// Additional constants needed
const HAL_MAX_DELAY: u32 = 0xFFFFFFFF;

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
        ptr::write_volatile(SYST_RVR as *mut u32, reload_value);
        ptr::write_volatile(SYST_CVR as *mut u32, 0);
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

fn i2s_wait<F>(test_flag: F, timeout: u32) -> HalStatus
where
    F: Fn() -> bool,
{
    let tick_start = hal_get_tick();

    while !test_flag() {
        let elapsed = hal_get_tick();
        let elapsed = elapsed.wrapping_sub(tick_start);

        if timeout != HAL_MAX_DELAY && elapsed > timeout {
            return HalStatus::Timeout;
        }
    }

    HalStatus::Ok
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
    for sample in p_data {
        // Wait until TXE flag is set
        if i2s_wait(|| hi2s.regs.sr().read().txe(), timeout) != HalStatus::Ok {
            // Set the error code and state are already set by the timeout function
            return HalStatus::Timeout;
        }

        // Write data to DR register
        hi2s.regs.dr().write(|w| w.set_dr(*sample));
    }

    // Wait until Busy flag is reset
    // XXX In the C code, this is only done when in SLAVE_TX or SLAVE_RX mode
    if i2s_wait(|| !hi2s.regs.sr().read().bsy(), timeout) != HalStatus::Ok {
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
    let max_size = p_tx_data.len().max(p_rx_data.len());

    if hi2s.state != HalI2sState::Ready {
        return HalStatus::Busy;
    }

    if p_tx_data.is_empty() || p_rx_data.is_empty() {
        return HalStatus::Error;
    }

    // Process Locked
    hi2s.lock = HalLock::Locked;

    let mut rx_data_ptr = p_rx_data.as_mut_ptr();

    // Set state and reset error code
    hi2s.error_code = HAL_I2S_ERROR_NONE;
    hi2s.state = HalI2sState::BusyTxRx;

    // Get the I2S mode configuration
    let i2s_mode = match hi2s.regs.i2scfgr().read().i2scfg() {
        I2scfg::SLAVE_TX => Mode::SlaveTx,
        I2scfg::SLAVE_RX => Mode::SlaveRx,
        I2scfg::MASTER_TX => Mode::MasterTx,
        I2scfg::MASTER_RX => Mode::MasterRx,
    };

    // Determine extended instance address
    let base_instance = hi2s.regs.as_ptr() as u32;
    let ext_instance = hi2s.regs_ext.as_ptr() as u32;

    // Check if the I2S_MODE_MASTER_TX or I2S_MODE_SLAVE_TX Mode is selected
    if (i2s_mode == Mode::MasterTx) || (i2s_mode == Mode::SlaveTx) {
        // Prepare the First Data before enabling the I2S
        hi2s.regs.dr().write(|w| w.set_dr(p_tx_data[0]));

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
        for i in 0..max_size {
            // Transmit data if available
            if i < p_tx_data.len() {
                // Wait until TXE flag is set on main instance
                if i2s_wait(|| hi2s.regs.sr().read().txe(), timeout) != HalStatus::Ok {
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Write Data on DR register of main instance
                hi2s.regs.dr().write(|w| w.set_dr(p_tx_data[i]));

                // Check if an underrun occurs (only for slave TX mode)
                if i2s_mode == Mode::SlaveTx {
                    // XXX unnecessary read
                    let _sr = unsafe { ptr::read_volatile((base_instance + 0x08) as *const u32) };
                    if hi2s.regs.sr().read().udr() {
                        // Clear Underrun flag
                        let _ = hi2s.regs.sr().read();
                        hi2s.error_code |= HAL_I2S_ERROR_UDR;
                    }
                }
            }

            // Receive data if available
            if i < p_rx_data.len() {
                // Wait until RXNE flag is set on extended instance
                if i2s_wait(|| hi2s.regs_ext.sr().read().rxne(), timeout) != HalStatus::Ok {
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
        hi2s.regs_ext.dr().write(|w| w.set_dr(p_tx_data[0]));

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
        let max_size = p_tx_data.len().max(p_rx_data.len());
        for i in 0..max_size {
            // Transmit data if available (use extended instance)
            if i < p_tx_data.len() - 1 {
                // Wait until TXE flag is set on extended instance
                if i2s_wait(|| hi2s.regs_ext.sr().read().txe(), timeout) != HalStatus::Ok {
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Write Data on DR register of extended instance
                hi2s.regs_ext.dr().write(|w| w.set_dr(p_tx_data[i + 1]));

                // Check if an underrun occurs on extended instance (only for slave RX mode)
                if i2s_mode == Mode::SlaveRx {
                    // XXX unnecessary read
                    let _sr_ext =
                        unsafe { ptr::read_volatile((ext_instance + 0x08) as *const u32) };
                    if !hi2s.regs_ext.sr().read().udr() {
                        // Clear Underrun flag
                        unsafe {
                            let _dummy = ptr::read_volatile((ext_instance + 0x08) as *const u32);
                        }
                        hi2s.error_code |= HAL_I2S_ERROR_UDR;
                    }
                }
            }

            // Receive data if available (use main instance)
            if i < p_rx_data.len() {
                // Wait until RXNE flag is set on main instance
                if i2s_wait(|| hi2s.regs.sr().read().rxne(), timeout) != HalStatus::Ok {
                    hi2s.error_code = HAL_I2S_ERROR_TIMEOUT;
                    hi2s.state = HalI2sState::Ready;
                    hi2s.lock = HalLock::Unlocked;
                    return HalStatus::Error;
                }

                // Read Data from DR register of main instance
                let rx_data = unsafe { ptr::read_volatile((base_instance + 0x0C) as *const u16) };
                unsafe {
                    ptr::write(rx_data_ptr, rx_data);
                }
                rx_data_ptr = unsafe { rx_data_ptr.add(1) };

                // Check if an overrun occurs on main instance
                // XXX This read should be unnecessary, but things lock up without it
                let _sr = unsafe { ptr::read_volatile((base_instance + 0x08) as *const u32) };
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
