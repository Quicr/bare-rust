use defmt::Format as DefmtFormat;
use embassy_stm32::{
    dma::{
        word::Word, AnyChannel, ReadableRingBuffer, Request, TransferOptions, WritableRingBuffer,
    },
    gpio::{AfType, Flex, OutputType, Pin, Speed},
    i2s::{ClockPolarity, Format, Standard},
    pac::spi::{vals::*, Spi},
    peripherals::{PB4, RCC},
    spi::{CkPin, MisoPin, MosiPin, RxDma, TxDma, WsPin},
    time::Hertz,
    Peri,
};
use num_enum::IntoPrimitive;

struct ChannelAndRequest<'d> {
    pub channel: Peri<'d, AnyChannel>,
    pub request: Request,
}

macro_rules! new_dma {
    ($name:ident) => {{
        let dma = $name;
        dma.remap();
        let request = dma.request();
        defmt::info!("dma request: {}", request);
        Some(ChannelAndRequest {
            channel: dma.into(),
            request,
        })
    }};
}

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
    Overrun,
    NotATransmitter,
    NotAReceiver,
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
        if let Some(timeout) = timeout {
            let elapsed = tick_start.elapsed().as_millis() as u32;
            if elapsed > timeout {
                return Err(Error::Timeout);
            }
        }
    }

    Ok(())
}

pub struct I2Sext<'d, T, W: Word = u16>
where
    T: embassy_stm32::spi::Instance + I2sRegs,
{
    regs: Spi,
    regs_ext: Spi,

    // DMA ring buffers for async I/O
    tx_ring_buffer: Option<WritableRingBuffer<'d, W>>,
    rx_ring_buffer: Option<ReadableRingBuffer<'d, W>>,

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

impl<'d, T, W: Word> I2Sext<'d, T, W>
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

        let af_num = SDEXT::af_num(&sd_ext);
        let mut sd_ext = Flex::new(sd_ext);
        sd_ext.set_as_af_unchecked(af_num, AfType::output(OutputType::PushPull, Speed::Low));

        let (regs, regs_ext) = T::get_regs();

        Self {
            regs,
            regs_ext,
            tx_ring_buffer: None,
            rx_ring_buffer: None,
            spi,
            ws,
            ck,
            sd,
            sd_ext,
        }
    }

    /// Create a new I2S extended peripheral with DMA support for full-duplex operation.
    ///
    /// This constructor sets up both TX and RX DMA ring buffers for continuous audio streaming.
    /// The TX DMA uses the main I2S data register, while RX DMA uses the extended I2S data register.
    pub fn new_with_dma<WS, CK, SD, SDEXT, TXDMA, RXDMA>(
        spi: Peri<'d, T>,
        ws: Peri<'d, WS>,
        ck: Peri<'d, CK>,
        sd: Peri<'d, SD>,
        sd_ext: Peri<'d, SDEXT>,
        txdma: Peri<'d, TXDMA>,
        tx_buffer: &'d mut [W],
        rxdma: Peri<'d, RXDMA>,
        rx_buffer: &'d mut [W],
    ) -> Self
    where
        WS: WsPin<T>,
        CK: CkPin<T>,
        SD: MosiPin<T>,
        SDEXT: SdExtPin<T> + MisoPin<T>,
        TXDMA: TxDma<T>,
        RXDMA: RxDma<T>,
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

        let af_num = <SDEXT as SdExtPin<T>>::af_num(&sd_ext);
        let mut sd_ext = Flex::new(sd_ext);
        sd_ext.set_as_af_unchecked(af_num, AfType::output(OutputType::PushPull, Speed::Low));

        let (regs, regs_ext) = T::get_regs();

        // Set up DMA transfer options
        let mut opts = TransferOptions::default();
        opts.half_transfer_ir = true;
        opts.complete_transfer_ir = true;
        opts.circular = true;

        // Configure DMA channels
        let txdma = new_dma!(txdma).map(|d| (d, tx_buffer));
        let rxdma = new_dma!(rxdma).map(|d| (d, rx_buffer));

        // Create TX ring buffer (uses main I2S DR)
        // TX uses DMA1_Stream7, Channel 0 (from Embassy's SPI3 TxDma trait)
        let tx_ptr = regs.dr().as_ptr() as *mut W;
        let tx_ring_buffer = txdma.map(|(ch, buf)| unsafe {
            WritableRingBuffer::new(ch.channel, ch.request, tx_ptr, buf, opts)
        });

        // Create RX ring buffer (uses extended I2S DR)
        // RX uses DMA1_Stream0, Channel 3 (I2S3ext, not SPI3!)
        // Embassy's SPI3 RxDma uses Channel 0, but I2S3ext needs Channel 3
        let rx_ptr = regs_ext.dr().as_ptr() as *mut W;
        let rx_ring_buffer = rxdma.map(|(ch, buf)| unsafe {
            let correct_request = 3u8; // I2S3ext uses DMA channel 3
            defmt::info!(
                "Overriding RX DMA request from {} to {}",
                ch.request,
                correct_request
            );
            ReadableRingBuffer::new(ch.channel, correct_request, rx_ptr, buf, opts)
        });

        Self {
            regs,
            regs_ext,
            tx_ring_buffer,
            rx_ring_buffer,
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

    /// Start DMA transfers for both TX and RX.
    /// This enables the DMA ring buffers and sets the DMA request enable bits on the I2S peripherals.
    /// After calling this, the I2S will continuously transfer audio data in the background.
    pub fn start(&mut self) {
        // Match STM HAL sequence for TX mode:
        // 1. Enable RX DMA first
        // 2. Enable RX DMA request on extended I2S
        // 3. Enable TX DMA
        // 4. Enable TX DMA request on main I2S
        // 5. Enable extended I2S (receiver)
        // 6. Enable main I2S (transmitter)

        if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
            defmt::info!("Starting RX ring buffer");
            rx_ring_buffer.start();
            // Enable RX DMA request on extended I2S peripheral
            self.regs_ext.cr2().modify(|w| w.set_rxdmaen(true));
            defmt::info!("RX DMA enabled, CR2={:08x}", self.regs_ext.cr2().read().0);
        }

        if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
            defmt::info!("Starting TX ring buffer");
            tx_ring_buffer.start();
            // Enable TX DMA request on main I2S peripheral
            self.regs.cr2().modify(|w| w.set_txdmaen(true));
            defmt::info!("TX DMA enabled, CR2={:08x}", self.regs.cr2().read().0);
        }

        // Enable both I2S peripherals (extended first, then main)
        self.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));
        defmt::info!(
            "I2S ext enabled, I2SCFGR={:08x}",
            self.regs_ext.i2scfgr().read().0
        );

        self.regs.i2scfgr().modify(|w| w.set_i2se(true));
        defmt::info!(
            "I2S main enabled, I2SCFGR={:08x}",
            self.regs.i2scfgr().read().0
        );

        // Check status registers
        defmt::info!(
            "I2S SR={:08x}, I2Sext SR={:08x}",
            self.regs.sr().read().0,
            self.regs_ext.sr().read().0
        );
    }

    /// Stop DMA transfers for both TX and RX.
    /// This waits for ongoing transfers to complete, then disables the DMA and I2S peripherals.
    pub async fn stop(&mut self) {
        let tx_stop = async {
            if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
                tx_ring_buffer.stop().await;
                self.regs.cr2().modify(|w| w.set_txdmaen(false));
            }
        };

        let rx_stop = async {
            if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
                rx_ring_buffer.stop().await;
                self.regs_ext.cr2().modify(|w| w.set_rxdmaen(false));
            }
        };

        embassy_futures::join::join(rx_stop, tx_stop).await;

        // Disable I2S peripherals
        self.regs.i2scfgr().modify(|w| w.set_i2se(false));
        self.regs_ext.i2scfgr().modify(|w| w.set_i2se(false));

        self.clear();
    }

    /// Clear/reset the DMA ring buffers to their initial state.
    /// This can be used to recover from overrun conditions.
    pub fn clear(&mut self) {
        if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
            tx_ring_buffer.clear();
        }
        if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
            rx_ring_buffer.clear();
        }
    }

    /// Check how much data is available in the RX ring buffer (for debugging)
    pub fn rx_available(&mut self) -> Result<usize, &str> {
        let Some(rx_ring_buffer) = self.rx_ring_buffer.as_mut() else {
            return Err("No ring buffer available");
        };

        match rx_ring_buffer.len() {
            Ok(len) => Ok(len),
            Err(err) => {
                defmt::error!("ring buffer error: {:?}", err);
                Err("ring buffer error")
            }
        }
    }

    /// Check status registers (for debugging)
    pub fn check_status(&self) {
        defmt::info!(
            "Main I2S SR={:08x} (TXE={} BSY={} UDR={} OVR={})",
            self.regs.sr().read().0,
            self.regs.sr().read().txe(),
            self.regs.sr().read().bsy(),
            self.regs.sr().read().udr(),
            self.regs.sr().read().ovr()
        );

        defmt::info!(
            "Ext I2S SR={:08x} (RXNE={} BSY={} UDR={} OVR={})",
            self.regs_ext.sr().read().0,
            self.regs_ext.sr().read().rxne(),
            self.regs_ext.sr().read().bsy(),
            self.regs_ext.sr().read().udr(),
            self.regs_ext.sr().read().ovr()
        );
    }

    /// Perform synchronized full-duplex I2S transfer via DMA.
    ///
    /// This method transmits data from `tx_data` while simultaneously receiving into `rx_data`.
    /// The buffers must be the same length. This matches the STM HAL's `HAL_I2SEx_TransmitReceive_DMA()`
    /// behavior where TX and RX are synchronized for each audio frame.
    ///
    /// # Arguments
    /// * `tx_data` - Data to transmit
    /// * `rx_data` - Buffer to receive data into
    ///
    /// # Errors
    /// Returns an error if:
    /// - DMA is not configured (ring buffers are None)
    /// - Buffer lengths don't match
    /// - DMA overrun/underrun occurs
    pub async fn transmit_receive_dma(
        &mut self,
        tx_data: &[W],
        rx_data: &mut [W],
    ) -> Result<(), Error> {
        if tx_data.len() != rx_data.len() {
            return Err(Error::EmptyBuffer); // Reuse this error for length mismatch
        }

        match (&mut self.tx_ring_buffer, &mut self.rx_ring_buffer) {
            (Some(tx_ring), Some(rx_ring)) => {
                // In full-duplex mode, we need to coordinate TX and RX
                // The STM HAL only uses RX DMA callbacks to drive both operations
                // We'll use embassy_futures::join to do both operations concurrently

                let tx_future = async {
                    let rv = tx_ring.write_exact(tx_data).await;
                    defmt::info!("tx_future done: {:?}", rv);
                    rv
                };
                let rx_future = async {
                    let rv = rx_ring.read_exact(rx_data).await;
                    defmt::info!("rx_future done: {:?}", rv);
                    rv
                };

                // Execute both DMA operations concurrently
                let (tx_result, rx_result) =
                    embassy_futures::join::join(tx_future, rx_future).await;

                tx_result.map_err(|_| Error::Overrun)?;
                rx_result.map_err(|_| Error::Overrun)?;

                Ok(())
            }
            (None, _) => Err(Error::NotATransmitter),
            (_, None) => Err(Error::NotAReceiver),
        }
    }

    /// Read data from the I2S receive ring buffer.
    ///
    /// **Note**: For full-duplex operation, prefer using `transmit_receive_dma()` instead,
    /// as it ensures TX and RX stay synchronized per the I2S protocol.
    ///
    /// This will wait asynchronously until the requested amount of data is available.
    /// The I2S is continuously receiving data in the background via DMA.
    pub async fn read(&mut self, data: &mut [W]) -> Result<(), Error> {
        match &mut self.rx_ring_buffer {
            Some(ring) => {
                ring.read_exact(data).await.map_err(|_| Error::Overrun)?;
                Ok(())
            }
            None => Err(Error::NotAReceiver),
        }
    }

    /// Write data to the I2S transmit ring buffer.
    ///
    /// **Note**: For full-duplex operation, prefer using `transmit_receive_dma()` instead,
    /// as it ensures TX and RX stay synchronized per the I2S protocol.
    ///
    /// This will wait asynchronously if there's not enough space in the buffer.
    /// The I2S is continuously transmitting data in the background via DMA.
    pub async fn write(&mut self, data: &[W]) -> Result<(), Error> {
        match &mut self.tx_ring_buffer {
            Some(ring) => {
                ring.write_exact(data).await.map_err(|_| Error::Overrun)?;
                Ok(())
            }
            None => Err(Error::NotATransmitter),
        }
    }
}
