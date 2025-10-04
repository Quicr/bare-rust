use defmt::Format as DefmtFormat;
use embassy_stm32::{
    dma::{
        word::Word, AnyChannel, ReadableRingBuffer, Request, TransferOptions, WritableRingBuffer,
    },
    gpio::{AfType, Flex, OutputType, Pin, Speed},
    i2s::{ClockPolarity, Format, Standard},
    pac::spi::{vals::*, Spi},
    peripherals::{PB4, RCC},
    spi::{CkPin, MosiPin, RxDma, TxDma, WsPin},
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
    pub fn new(
        spi: Peri<'d, T>,
        ws: Peri<'d, impl WsPin<T>>,
        ck: Peri<'d, impl CkPin<T>>,
        sd: Peri<'d, impl MosiPin<T>>,
        sd_ext: Peri<'d, impl SdExtPin<T>>,
        txdma: Peri<'d, impl TxDma<T>>,
        tx_buffer: &'d mut [W],
        rxdma: Peri<'d, impl RxDma<T>>,
        rx_buffer: &'d mut [W],
    ) -> Self {
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
            // XXX This is a bug in Embassy, or perhaps a mismatch between Embassy's targeting
            // SPI and our targeting I2S3ext.  These numbers come from spi::TxDma; we may want to
            // do something like define a TxDmaExt trait (similar to the SdExtPin), that would
            // perform this override.
            let correct_request = 3u8;
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

    pub fn start(&mut self) {
        if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
            rx_ring_buffer.start();
            // Enable RX DMA request on extended I2S peripheral
            self.regs_ext.cr2().modify(|w| w.set_rxdmaen(true));
        }

        if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
            tx_ring_buffer.start();
            // Enable TX DMA request on main I2S peripheral
            self.regs.cr2().modify(|w| w.set_txdmaen(true));
        }

        // Enable both I2S peripherals (extended first, then main)
        self.regs_ext.i2scfgr().modify(|w| w.set_i2se(true));
        self.regs.i2scfgr().modify(|w| w.set_i2se(true));
    }

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

    pub fn clear(&mut self) {
        if let Some(tx_ring_buffer) = &mut self.tx_ring_buffer {
            tx_ring_buffer.clear();
        }
        if let Some(rx_ring_buffer) = &mut self.rx_ring_buffer {
            rx_ring_buffer.clear();
        }
    }

    pub async fn transmit_receive(
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

                let tx_future = tx_ring.write_exact(tx_data);
                let rx_future = rx_ring.read_exact(rx_data);

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
}
