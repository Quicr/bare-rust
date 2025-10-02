#![no_std]
#![no_main]
#![allow(dead_code)] // XXX

mod board;
mod hal_i2s;
use hal_i2s::*;

use board::{AudioControl, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::Event;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{mode::Async, usart::UartRx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;
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
async fn main(_spawner: Spawner) {
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
    let i2c = embassy_stm32::i2c::I2c::new_blocking(p.I2C1, p.PB6, p.PB7, config);
    let mut audio_control = AudioControl::new(i2c);
    audio_control.init();

    // HAL_I2S_MspInit() - Configure I2S3 GPIO and clocks
    let _ = hal_i2s_msp_init(p.SPI3, p.PA15, p.PC10, p.PB5, p.PB4);

    // MX_I2S3_Init() - Configure I2S3 parameters
    let config = {
        use embassy_stm32::i2s::Format;
        use hal_i2s::Config;

        let mut config = Config::default();
        config.mode = Mode::SlaveTx;
        config.standard = Standard::Philips;
        config.data_format = Format::Data16Channel32;
        config.master_clock = false;
        config.audio_freq = AudioFreq::Hz8k;
        config.cpol = Cpol::Low;
        config.clock_source = ClockSource::Plli2s;
        config.full_duplex_mode = FullDuplexMode::Enable;
        config
    };

    let mut i2s = I2sHandle::new_spi3();
    i2s.init(config).expect("Failed to initialize I2S");

    let square_frame: [u16; 16_000] = core::array::from_fn(|i| {
        const LAMBDA: u16 = 18; // Generates 444hz at 8khz sample rate
        const AMPLITUDE: u16 = 0x1fff;

        (((i as u16) / LAMBDA) % 2) * AMPLITUDE
    });

    trace!("before tx");
    i2s.transmit(&square_frame, Some(100))
        .expect("Failed to transmit");
    trace!("after tx");

    trace!("before txrx");
    let mut last_frame = [0; 16_000];
    let mut curr_frame = [0; 16_000];
    loop {
        i2s.transmit_receive(&last_frame, &mut curr_frame, Some(100))
            .expect("Failed to transmit/receive");

        last_frame.copy_from_slice(&curr_frame);
    }
}
