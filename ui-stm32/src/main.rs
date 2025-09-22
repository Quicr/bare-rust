#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    i2c::{mode::Master, Error, I2c},
    mode::{Async, Blocking},
    usart::UartRx,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;
use hex::ToHex;
use {defmt_rtt as _, panic_probe as _};

// Configuration parameters
const EVENT_QUEUE_DEPTH: usize = 10;
const KEYBOARD_SCAN_MILLIS: u64 = 50;

type EventChannel = Channel<CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;
type EventSender = Sender<'static, CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;

static EVENT_QUEUE: EventChannel = Channel::new();

#[embassy_executor::task(pool_size = 2)]
async fn monitor_button(mut button: Button, id: ButtonId, events: EventSender) {
    loop {
        button.wait_for_rising_edge().await;
        events.send(Event::ButtonDown(id)).await;
        button.wait_for_falling_edge().await;
        events.send(Event::ButtonUp(id)).await;
    }
}

#[embassy_executor::task]
async fn monitor_keyboard(mut keyboard: Keyboard, events: EventSender) {
    loop {
        let _ = Timer::after_millis(KEYBOARD_SCAN_MILLIS).await;
        for event in keyboard.scan() {
            events.send(event).await;
        }
    }
}

#[embassy_executor::task]
async fn monitor_net(from: UartRx<'static, Async>, events: EventSender) {
    const DMA_BUFFER_SIZE: usize = 1024;

    // Wrap the raw receiver in a DMA-buffered, SLIP-parsing, TLV-parsing version
    let mut dma_buf = [0u8; DMA_BUFFER_SIZE];
    let mut from = from.into_ring_buffered(&mut dma_buf);
    let mut from = NetRx::new(&mut from);

    loop {
        let Some(from_net) = from.next().await else {
            continue;
        };

        events.send(Event::FromNet(from_net)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /*
    info!("about to instantiate board");

    let mut board = Board::new().await;
    let mut app = App::new();

    info!("done setting up board and app");

    // Capture button events
    unwrap!(spawner.spawn(monitor_button(
        board.button_a.take().unwrap(),
        ButtonId::A,
        EVENT_QUEUE.sender()
    )));

    unwrap!(spawner.spawn(monitor_button(
        board.button_b.take().unwrap(),
        ButtonId::B,
        EVENT_QUEUE.sender()
    )));

    // Capture keyboard events
    unwrap!(spawner.spawn(monitor_keyboard(
        board.keyboard.take().unwrap(),
        EVENT_QUEUE.sender()
    )));

    // Capture UART events from the NET chip
    unwrap!(spawner.spawn(monitor_net(
        board.net_rx.take().unwrap(),
        EVENT_QUEUE.sender()
    )));

    debug!("app start");
    app.start(&mut board);

    // Main event loop
    loop {
        let event = EVENT_QUEUE.receive().await;
        app.handle(event, &mut board);
    }
    */

    /*
    ///// EEPROM Demo /////

    // Read the current contents of the EEPROM
    const I2C_ADDR: u8 = 0x50;
    const ADDR: u8 = 0x00;

    let mut data = [0u8; 256];
    board
        .i2c
        .blocking_write_read(I2C_ADDR, &[ADDR], &mut data)
        .unwrap();
    let hex: heapless::String<1024> = data.encode_hex();
    info!("eeprom before {}", hex);

    // Overwrite the EEPROM with a new value
    let mut data = [0xA0; 17];
    for i in (0_u8..=0xff).step_by(16) {
        data[0] = i;
        board.i2c.blocking_write(I2C_ADDR, &data).unwrap();
        Timer::after_millis(10).await;
    }

    // Read the value back out of the EEPROM
    let mut data = [0u8; 256];
    board
        .i2c
        .blocking_write_read(I2C_ADDR, &[ADDR], &mut data)
        .unwrap();
    let hex: heapless::String<1024> = data.encode_hex();
    info!("eeprom after {}", hex);
    */

    ///// Audio Chip /////
    let config = {
        use embassy_stm32::{rcc::*, time::Hertz};

        let mut config = embassy_stm32::Config::default();

        config.rcc.hse = Some(Hse {
            freq: Hertz(6_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV3,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2),
            divq: Some(PllQDiv::DIV7),
            divr: None,
        });

        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.ls = LsConfig {
            rtc: RtcClockSource::LSI,
            lsi: true,
            lse: None,
        };

        // XXX(RLB) The prediv = M value here must be the same as the PLL config above.  The
        // CubeMX clock tree shows one M value for both PLLs.
        config.rcc.plli2s = Some(Pll {
            prediv: PllPreDiv::DIV3,
            mul: PllMul::MUL50,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2),
        });

        config
    };
    let p = embassy_stm32::init(config);

    // Do audio chip setup over I2C
    let config = {
        use embassy_stm32::{gpio::Speed, i2c::*, time::Hertz};

        let mut config = Config::default();

        config.frequency = Hertz(100_000);
        config.gpio_speed = Speed::VeryHigh;
        config.sda_pullup = false;
        config.scl_pullup = false;
        config.timeout = embassy_time::Duration::from_millis(1000);

        config
    };
    let i2c = embassy_stm32::i2c::I2c::new_blocking(p.I2C1, p.PB6, p.PB7, Default::default());
    let mut audio_control = AudioControl::new(i2c);
    audio_control.init().await;

    // Receive audio over I2S
    const I2S_BUFFER_SIZE: usize = 8000;
    let mut rx_buffer = [0u16; I2S_BUFFER_SIZE];
    let mut tx_buffer = [0u16; I2S_BUFFER_SIZE];

    // Play out the recorded audio
    let mut config = {
        use embassy_stm32::{i2s::*, time::Hertz};

        let mut config = Config::default();
        config.frequency = Hertz(8_000);
        config.mode = Mode::Slave;
        config.standard = Standard::Philips;
        config.format = Format::Data16Channel32;
        config.clock_polarity = ClockPolarity::IdleLow;
        config.master_clock = false;
        config
    };
    let mut i2s = embassy_stm32::i2s::I2S::new_full_duplex_ex(
        p.SPI3,         // peri
        p.PB5,          // txsd
        p.PB4,          // rxsd
        p.PA15,         // ws
        p.PC10,         // ck
        p.DMA1_CH7,     // txdma
        &mut tx_buffer, // txdma_buf
        p.DMA1_CH0,     // rxdma
        &mut rx_buffer, // rxdma_buf
        config,
    );

    i2s.start();

    // Play a 444Hz square wave => 18 samples per cycle
    let square_wave: [u16; 108] = [
        0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff,
        0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff,
        0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff,
        0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff,
        0x1fff, 0x1fff, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ];
    trace!("before beep");
    for _i in 0..(16_000 / square_wave.len()) {
        i2s.write(&square_wave).await.ok();
    }
    trace!("after beep");

    trace!("starting loopback...");
    let mut chunk = [0u16; 160]; // 20ms chunks
    loop {
        trace!("awaiting chunk");
        i2s.read(&mut chunk).await.unwrap();
        trace!("chunk {}", chunk.iter().sum::<u16>());
        // i2s.write(&chunk).await.unwrap();
        break;
    }

    // i2s.stop().await;
}

struct AudioControl {
    i2c: I2c<'static, Blocking, Master>,
    r: [u16; 128],
}

impl AudioControl {
    const VALUE_MASK: u16 = 0x1ff;

    fn new(i2c: I2c<'static, Blocking, Master>) -> Self {
        Self { i2c, r: [0; 128] }
    }

    async fn init(&mut self) {
        // Reset the wm8960
        self.set_register(0x0F, 0b1_0000_0000);
        Timer::after_millis(100).await;

        // Set the power
        self.set_register(0x19, 0b0_1111_1110);

        // Enable outputs
        self.set_register(0x1A, 0b1_1110_0001);

        // Enable lr mixer ctrl
        // self.set_register(0x2F, 0b0_0000_0000);
        self.set_register(0x2F, 0b0_0010_1100);

        // Disable soft mute and ADC high pass filter
        self.set_register(0x05, 0b0_0000_0000);

        // Set clocks for 8kHz
        self.set_register(0x34, 0b0_0000_1000);
        self.set_register(0x35, 0b0_0011_0001);
        self.set_register(0x36, 0b0_0010_0110);
        self.set_register(0x37, 0b0_1110_1001);
        self.set_register(0x04, 0b1_1011_0001);
        self.set_register(0x08, 0b1_1100_1100);
        self.set_register(0x1B, 0b0_0000_0101);

        // Set mono
        self.set_bit(0x17, 4, true);
        self.set_bit(0x2A, 6, false);

        // Set volumes
        const DEFAULT_VOLUME: u16 = 0b110_0111;
        const DEFAULT_MIC_VOLUME: u16 = 0b11_1111;
        self.set_bits(0x00, 0b1_0011_1111, 0x100 + DEFAULT_MIC_VOLUME);
        self.set_bits(0x02, 0b1_0111_1111, DEFAULT_VOLUME);
        self.set_bits(0x03, 0b1_0111_1111, 0x100 + DEFAULT_VOLUME);

        // Enable the outputs
        self.set_register(0x31, 0b0_0111_0111);

        // Set DAC left and right volumes
        self.set_register(0x0A, 0b1_1111_1111);
        self.set_register(0x0B, 0b1_1111_1111);

        // Set left and right mixer
        self.set_register(0x22, 0b1_0000_0000);
        self.set_register(0x25, 0b1_0000_0000);

        self.set_bits(0x2B, 0b0_0111_0000, 0b0_0111_0000); // XXX extra 0; typo in C?

        // Enable DAC softmute
        self.set_bit(0x06, 3, true);

        // Set the Master mode (1), I2S to 16 bit words
        // Set audio data format to i2s mode
        self.set_register(0x07, 0b0_0100_0010);

        // Unmute the mic
        self.set_bit(0x20, 6, false);
        self.set_bit(0x20, 8, false);
        self.set_bit(0x19, 5, true);
        self.set_bit(0x2f, 5, true);
        self.set_bit(0x20, 3, true);
        self.set_bit(0x20, 7, false);
        self.set_bit(0x20, 6, true);
        self.set_bit(0x20, 8, true);
        self.set_bits(0x00, 0b1_1000_0000, 0b1_0000_0000);
        self.set_bits(0x2B, 0b0_0000_1110, 0b0_0000_1010);
        self.set_bit(0x19, 1, true);
    }

    fn set_register(&mut self, addr: u8, value: u16) {
        self.r[addr as usize] = value & Self::VALUE_MASK;
        self.write_register(addr);
    }

    fn set_bit(&mut self, addr: u8, which: usize, value: bool) {
        defmt::assert!(which < 9);
        let mask = 1 << which;
        let value: u16 = value.into();
        self.r[addr as usize] = (self.r[addr as usize] & !mask) | (value << which);
        self.write_register(addr);
    }

    fn set_bits(&mut self, addr: u8, mask: u16, value: u16) {
        defmt::assert_eq!(mask & !Self::VALUE_MASK, 0);
        defmt::assert_eq!(!mask & value, 0);
        self.r[addr as usize] = (self.r[addr as usize] & !mask) | (mask & value);
        self.write_register(addr);
    }

    fn write_register(&mut self, addr: u8) {
        const ADDR_MASK: u16 = 0x7f;
        const VALUE_MASK: u16 = 0x1ff;
        const DEVICE_ADDR: u8 = 0x1a;

        let to_write = (((addr as u16) & ADDR_MASK) << 9) | (self.r[addr as usize] & VALUE_MASK);
        self.i2c
            .blocking_write(DEVICE_ADDR, &to_write.to_be_bytes())
            .unwrap();
    }
}
