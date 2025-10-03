use defmt::Format as DefmtFormat;
use embassy_stm32::gpio::Pin;
use embassy_stm32::gpio::{AfType, Flex, OutputType, Speed};
use embassy_stm32::i2s::{ClockPolarity, Format, Standard};
use embassy_stm32::pac::spi::{vals::*, Spi};
use embassy_stm32::peripherals::PB4;
use embassy_stm32::peripherals::RCC;
use embassy_stm32::spi::CkPin;
use embassy_stm32::spi::MosiPin;
use embassy_stm32::spi::WsPin;
use embassy_stm32::time::Hertz;
use embassy_stm32::Peri;
use num_enum::IntoPrimitive;

// These mapping functions are copy/pasted private methods from i2s.rs
const fn datlen(format: Format) -> Datlen {
    match format {
        Format::Data16Channel16 => Datlen::BITS16,
        Format::Data16Channel32 => Datlen::BITS16,
        Format::Data24Channel32 => Datlen::BITS24,
        Format::Data32Channel32 => Datlen::BITS32,
    }
}

const fn chlen(format: Format) -> Chlen {
    match format {
        Format::Data16Channel16 => Chlen::BITS16,
        Format::Data16Channel32 => Chlen::BITS32,
        Format::Data24Channel32 => Chlen::BITS32,
        Format::Data32Channel32 => Chlen::BITS32,
    }
}

const fn to_ckpol(clock_polarity: ClockPolarity) -> Ckpol {
    match clock_polarity {
        ClockPolarity::IdleLow => Ckpol::IDLE_LOW,
        ClockPolarity::IdleHigh => Ckpol::IDLE_HIGH,
    }
}

const fn to_i2sstd(standard: Standard) -> I2sstd {
    match standard {
        Standard::Philips => I2sstd::PHILIPS,
        Standard::MsbFirst => I2sstd::MSB,
        Standard::LsbFirst => I2sstd::LSB,
        Standard::PcmLongSync => I2sstd::PCM,
        Standard::PcmShortSync => I2sstd::PCM,
    }
}

const fn to_pcmsync(standard: Standard) -> Pcmsync {
    match standard {
        Standard::PcmLongSync => Pcmsync::LONG,
        _ => Pcmsync::SHORT,
    }
}

fn compute_baud_rate(
    i2s_clock: Hertz,
    request_freq: Hertz,
    mclk: bool,
    format: Format,
) -> (bool, u8) {
    let coef = if mclk {
        256
    } else if let Format::Data16Channel16 = format {
        32
    } else {
        64
    };

    let (n, d) = (i2s_clock.0, coef * request_freq.0);
    let division = (n + (d >> 1)) / d;

    if division < 4 {
        (false, 2)
    } else if division > 511 {
        (true, 255)
    } else {
        ((division & 1) == 1, (division >> 1) as u8)
    }
}

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
pub enum FullDuplexMode {
    Disable = 0x00000000,
    Enable = 0x00000001,
}

#[derive(DefmtFormat, Debug, Clone, Copy, PartialEq)]
pub enum Error {
    Busy,
    Timeout,
    InvalidPrescaler,
    EmptyBuffer,
}

/// Non-fatal errors that occurred during transfer
#[derive(Debug, Clone, Copy, Default)]
pub struct TransferErrors {
    pub underrun: bool,
    pub overrun: bool,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub mode: Mode,
    pub standard: Standard,
    pub format: Format,
    pub master_clock: bool,
    pub frequency: Hertz,
    pub clock_polarity: ClockPolarity,
    pub full_duplex_mode: FullDuplexMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::SlaveTx,
            standard: Standard::Philips,
            format: Format::Data16Channel16,
            master_clock: false,
            frequency: Hertz(8_000),
            clock_polarity: ClockPolarity::IdleLow,
            full_duplex_mode: FullDuplexMode::Disable,
        }
    }
}

const SPI2: Spi = unsafe { Spi::from_ptr(0x40003800 as *mut ()) };
const SPI3: Spi = unsafe { Spi::from_ptr(0x40003C00 as *mut ()) };
const I2S2EXT: Spi = unsafe { Spi::from_ptr(0x40003400 as *mut ()) };
const I2S3EXT: Spi = unsafe { Spi::from_ptr(0x40004000 as *mut ()) };

pub trait I2sRegs {
    fn enable_rcc();
    fn get_regs() -> (Spi, Spi);
}

impl I2sRegs for embassy_stm32::peripherals::SPI2 {
    fn enable_rcc() {
        use embassy_stm32::pac::RCC;
        RCC.apb1enr().modify(|w| w.set_spi2en(true));
    }

    fn get_regs() -> (Spi, Spi) {
        (SPI2, I2S2EXT)
    }
}

impl I2sRegs for embassy_stm32::peripherals::SPI3 {
    fn enable_rcc() {
        use embassy_stm32::pac::RCC;
        RCC.apb1enr().modify(|w| w.set_spi3en(true));
    }

    fn get_regs() -> (Spi, Spi) {
        (SPI3, I2S3EXT)
    }
}

trait EnableGpioPort {
    fn enable_gpio_port(&self);
}

impl<T> EnableGpioPort for T
where
    T: Pin,
{
    fn enable_gpio_port(&self) {
        use embassy_stm32::pac::RCC;
        RCC.ahb1enr().modify(|w| match self.port() {
            0 => w.set_gpioaen(true),
            1 => w.set_gpioben(true),
            2 => w.set_gpiocen(true),
            _ => unreachable!(),
        });
    }
}

pub trait SdExtPin<T>: Pin {
    fn af_num(&self) -> u8;
}

// SdExt assignment manually copied from the STM32F405RG data sheet (p. 64)
impl SdExtPin<embassy_stm32::peripherals::SPI3> for PB4 {
    #[inline(always)]
    fn af_num(&self) -> u8 {
        7
    }
}

fn i2s_wait<F>(test_flag: F, timeout: Option<u32>) -> Result<(), Error>
where
    F: Fn() -> bool,
{
    use embassy_time::Instant;

    let tick_start = Instant::now();

    while !test_flag() {
        let elapsed = tick_start.elapsed().as_millis() as u32;

        if let Some(timeout) = timeout
            && elapsed > timeout
        {
            return Err(Error::Timeout);
        }
    }

    Ok(())
}

pub struct I2Sext<'d, T>
where
    T: embassy_stm32::spi::Instance + I2sRegs,
{
    regs: Spi,
    regs_ext: Spi,

    // These fields are held just to keep them alive
    #[allow(dead_code)]
    spi: Peri<'d, T>,
    #[allow(dead_code)]
    ws: Flex<'d>,
    #[allow(dead_code)]
    ck: Flex<'d>,
    #[allow(dead_code)]
    sd: Flex<'d>,
    #[allow(dead_code)]
    sd_ext: Flex<'d>,
}

impl<'d, T> I2Sext<'d, T>
where
    T: embassy_stm32::spi::Instance + I2sRegs,
{
    pub fn new<WS, CK, SD, SDEXT>(
        spi: Peri<'d, T>,
        ws: Peri<'d, WS>,
        ck: Peri<'d, CK>,
        sd: Peri<'d, SD>,
        sd_ext: Peri<'d, SDEXT>,
    ) -> Self
    where
        WS: WsPin<T>,
        CK: CkPin<T>,
        SD: MosiPin<T>,
        SDEXT: SdExtPin<T>,
    {
        // Enable peripheral clocks
        T::enable_rcc();
        ws.enable_gpio_port();
        ck.enable_gpio_port();
        sd.enable_gpio_port();
        sd_ext.enable_gpio_port();

        // Configure the pins
        let af_num = ws.af_num();
        let mut ws = Flex::new(ws);
        ws.set_as_af_unchecked(af_num, AfType::output(OutputType::PushPull, Speed::Low));

        let af_num = ck.af_num();
        let mut ck = Flex::new(ck);
        ck.set_as_af_unchecked(af_num, AfType::output(OutputType::PushPull, Speed::Low));

        let af_num = sd.af_num();
        let mut sd = Flex::new(sd);
        sd.set_as_af_unchecked(af_num, AfType::output(OutputType::PushPull, Speed::Low));

        let af_num = sd_ext.af_num();
        let mut sd_ext = Flex::new(sd_ext);
        sd_ext.set_as_af_unchecked(af_num, AfType::output(OutputType::PushPull, Speed::Low));

        let (regs, regs_ext) = T::get_regs();

        Self {
            regs,
            regs_ext,
            spi,
            ws,
            ck,
            sd,
            sd_ext,
        }
    }

    pub fn init(&mut self, rcc: &Peri<RCC>, config: Config) -> Result<(), Error> {
        // I2SPR: I2SDIV and ODD Calculation
        let pclk = embassy_stm32::rcc::clocks(rcc)
            .plli2s1_r
            .to_hertz()
            .unwrap();

        let (odd, div) =
            compute_baud_rate(pclk, config.frequency, config.master_clock, config.format);

        // Write to SPIx I2SPR register the computed value
        self.regs.i2spr().modify(|w| {
            w.set_i2sdiv(div);
            w.set_odd(if odd { Odd::ODD } else { Odd::EVEN });
            w.set_mckoe(config.master_clock);
        });

        // Clear I2SMOD, I2SE, I2SCFG, PCMSYNC, I2SSTD, CKPOL, DATLEN and CHLEN bits
        // And configure the I2S with the InitStruct values
        self.regs.i2scfgr().modify(|w| {
            // TODO use semantic modifiers
            let mode: u32 = config.mode.into();
            w.0 = mode;
            w.set_i2smod(true);
            w.set_i2sstd(to_i2sstd(config.standard));
            w.set_pcmsync(to_pcmsync(config.standard));
            w.set_datlen(datlen(config.format));
            w.set_chlen(chlen(config.format));
            w.set_ckpol(to_ckpol(config.clock_polarity));
        });

        // Configure the I2S extended if the full duplex mode is enabled
        if config.full_duplex_mode == FullDuplexMode::Enable {
            // Get the mode to be configured for the extended I2S
            let ext_mode = match config.mode {
                Mode::MasterTx | Mode::SlaveTx => Mode::SlaveRx,
                Mode::MasterRx | Mode::SlaveRx => Mode::SlaveTx,
            };

            // Configure the I2S Slave with the I2S Master parameter values
            self.regs_ext.i2scfgr().modify(|w| {
                // TODO use semantic modifiers
                let mode: u32 = ext_mode.into();
                w.0 = mode;
                w.set_i2smod(true);
                w.set_i2sstd(to_i2sstd(config.standard));
                w.set_pcmsync(to_pcmsync(config.standard));
                w.set_datlen(datlen(config.format));
                w.set_chlen(chlen(config.format));
                w.set_ckpol(to_ckpol(config.clock_polarity));
            });
        }

        Ok(())
    }

    pub fn transmit(&mut self, p_data: &[u16], timeout: Option<u32>) -> Result<(), Error> {
        if p_data.is_empty() {
            return Err(Error::EmptyBuffer);
        }

        // Check if the I2S is already enabled
        self.regs.i2scfgr().modify(|w| {
            if !w.i2se() {
                w.set_i2se(true);
            }
        });

        // Start the transfer
        for sample in p_data {
            // Wait until TXE flag is set
            i2s_wait(|| self.regs.sr().read().txe(), timeout)?;

            // Write data to DR register
            self.regs.dr().write(|w| w.set_dr(*sample));
        }

        // Wait until Busy flag is reset
        // XXX In the C code, this is only done when in SLAVE_TX or SLAVE_RX mode
        i2s_wait(|| !self.regs.sr().read().bsy(), timeout)?;

        Ok(())
    }

    pub fn transmit_receive(
        &mut self,
        p_tx_data: &[u16],
        p_rx_data: &mut [u16],
        timeout: Option<u32>,
    ) -> Result<TransferErrors, Error> {
        let max_size = p_tx_data.len().max(p_rx_data.len());
        let mut errors = TransferErrors::default();

        if p_tx_data.is_empty() || p_rx_data.is_empty() {
            return Err(Error::EmptyBuffer);
        }

        // Get the I2S mode configuration
        let i2s_mode = match self.regs.i2scfgr().read().i2scfg() {
            I2scfg::SLAVE_TX => Mode::SlaveTx,
            I2scfg::SLAVE_RX => Mode::SlaveRx,
            I2scfg::MASTER_TX => Mode::MasterTx,
            I2scfg::MASTER_RX => Mode::MasterRx,
        };

        // Check if the I2S_MODE_MASTER_TX or I2S_MODE_SLAVE_TX Mode is selected
        if (i2s_mode == Mode::MasterTx) || (i2s_mode == Mode::SlaveTx) {
            // Prepare the First Data before enabling the I2S
            self.regs.dr().write(|w| w.set_dr(p_tx_data[0]));

            // Enable peripherals
            self.regs.i2scfgr().modify(|w| w.set_i2se(true));
            self.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));

            // Clear the Overrun Flag if in master TX mode
            if i2s_mode == Mode::MasterTx {
                // Clear overrun flag by reading DR then SR of extended instance
                let _ = self.regs_ext.dr().read();
                let _ = self.regs_ext.sr().read();
            }

            // Main transfer loop
            for i in 0..max_size {
                // Transmit data if available
                if i < p_tx_data.len() {
                    // Wait until TXE flag is set on main instance
                    i2s_wait(|| self.regs.sr().read().txe(), timeout)?;

                    // Write Data on DR register of main instance
                    self.regs.dr().write(|w| w.set_dr(p_tx_data[i]));

                    // Check if an underrun occurs (only for slave TX mode)
                    if i2s_mode == Mode::SlaveTx {
                        if self.regs.sr().read().udr() {
                            // Clear Underrun flag
                            let _ = self.regs.sr().read();
                            errors.underrun = true;
                        }
                    }
                }

                // Receive data if available
                if i < p_rx_data.len() {
                    // Wait until RXNE flag is set on extended instance
                    i2s_wait(|| self.regs_ext.sr().read().rxne(), timeout)?;

                    // Read Data from DR register of extended instance
                    let rx_data = self.regs_ext.dr().read().dr();
                    p_rx_data[i] = rx_data;

                    // Check if an overrun occurs on extended instance
                    if self.regs_ext.sr().read().ovr() {
                        // Clear Overrun flag
                        let _ = self.regs_ext.dr().read();
                        let _ = self.regs_ext.sr().read();
                        errors.overrun = true;
                    }
                }
            }
        } else {
            // Prepare the First Data before enabling the I2S (write to extended instance)
            self.regs_ext.dr().write(|w| w.set_dr(p_tx_data[0]));

            // Enable the peripherals
            self.regs.i2scfgr().modify(|w| w.set_i2se(true));
            self.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));

            // Clear the Overrun Flag if in master RX mode
            if i2s_mode == Mode::MasterRx {
                // Clear overrun flag by reading DR then SR of main instance
                let _ = self.regs.dr().read();
                let _ = self.regs.sr().read();
            }

            // Main transfer loop
            let max_size = p_tx_data.len().max(p_rx_data.len());
            for i in 0..max_size {
                // Transmit data if available (use extended instance)
                if i < p_tx_data.len() - 1 {
                    // Wait until TXE flag is set on extended instance
                    i2s_wait(|| self.regs_ext.sr().read().txe(), timeout)?;

                    // Write Data on DR register of extended instance
                    self.regs_ext.dr().write(|w| w.set_dr(p_tx_data[i + 1]));

                    // Check if an underrun occurs on extended instance (only for slave RX mode)
                    if i2s_mode == Mode::SlaveRx {
                        if !self.regs_ext.sr().read().udr() {
                            // Clear Underrun flag
                            let _ = self.regs_ext.sr().read();
                            errors.underrun = true;
                        }
                    }
                }

                // Receive data if available (use main instance)
                if i < p_rx_data.len() {
                    // Wait until RXNE flag is set on main instance
                    i2s_wait(|| self.regs.sr().read().rxne(), timeout)?;

                    // Read Data from DR register of main instance
                    let rx_data = self.regs.dr().read().dr();
                    p_rx_data[i] = rx_data;

                    // Check if an overrun occurs on main instance
                    if self.regs.sr().read().ovr() {
                        // Clear Overrun flag
                        let _ = self.regs.dr().read();
                        let _ = self.regs.sr().read();
                        errors.overrun = true;
                    }
                }
            }
        }

        Ok(errors)
    }
}
