#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::{App, AudioControl, Event, Outputs};

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{i2s::I2S, mode::Async, usart::UartRx};
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
async fn main(spawner: Spawner) {
    info!("about to instantiate board");

    const LAMBDA: usize = 18;
    const TARGET_FRAME_SIZE: usize = 4000;
    const FRAME_SIZE: usize = TARGET_FRAME_SIZE - (TARGET_FRAME_SIZE % (2 * LAMBDA));
    const BUFFER_SIZE: usize = 2 * FRAME_SIZE;

    let i2s_tx = singleton!(: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
    let i2s_rx = singleton!(: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();

    let mut board = Board::new(i2s_tx, i2s_rx);
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

    board.audio_control().start();

    let mut i2s = board.i2s.take().unwrap();

    // Main event loop
    /*
    loop {
        let event = EVENT_QUEUE.receive().await;
        app.handle(event, &mut board);
    }
    */

    /*
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
    let mut i2c = embassy_stm32::i2c::I2c::new_blocking(p.I2C1, p.PB6, p.PB7, config);
    let mut audio_control = AudioControl::new(&mut i2c);
    audio_control.init();

    // MX_I2S3_Init() - Configure I2S3 parameters
    let config = {
        use embassy_stm32::{
            i2s::{ClockPolarity, Config, Format, Mode, Standard},
            time::Hertz,
        };

        let mut config = Config::default();
        config.mode = Mode::Slave;
        config.standard = Standard::Philips;
        config.format = Format::Data16Channel32;
        config.master_clock = false;
        config.frequency = Hertz(8_000);
        config.clock_polarity = ClockPolarity::IdleLow;
        config
    };

    let mut i2s: I2S<u16> = I2S::new_full_duplex(
        p.SPI3,
        p.PA15,
        p.PC10,
        p.PB5,
        p.PB4,
        p.DMA1_CH7,
        &mut tx_buf,
        p.DMA1_CH0,
        &mut rx_buf,
        config.clone(),
    );
    i2s.start();
    */

    let square_frame: [u16; FRAME_SIZE] = core::array::from_fn(|i| {
        const AMPLITUDE: u16 = 0x1fff;
        (((i / LAMBDA) % 2) as u16) * AMPLITUDE
    });

    trace!("before tx");
    let mut last_frame = [0; FRAME_SIZE];
    let mut curr_frame = [0; FRAME_SIZE];
    for _i in 0..(16_000 / square_frame.len()) {
        i2s.read_write(&square_frame, &mut last_frame)
            .await
            .expect("Failed to transmit");
    }
    trace!("after tx");

    trace!("before txrx");
    loop {
        trace!(
            "tick {}",
            last_frame.iter().map(|x| *x as usize).sum::<usize>()
        );

        i2s.read_write(&last_frame, &mut curr_frame)
            .await
            .expect("Failed to transmit/receive");

        trace!(
            "t0ck {}",
            curr_frame.iter().map(|x| *x as usize).sum::<usize>()
        );

        last_frame.copy_from_slice(&curr_frame);
    }
}
