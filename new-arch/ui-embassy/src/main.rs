#![no_std]
#![no_main]

use ui_app::Event;

use core::fmt::Write;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::usart::{Config, DataBits, Parity, StopBits, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;
use heapless::{mpmc::Q64, String};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

// static EVENT_QUEUE: Q64<Event> = Q64::new();

const EVENT_QUEUE_DEPTH: usize = 10;
type EventChannel = Channel<CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;
type EventSender = Sender<'static, CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;

static EVENT_QUEUE: EventChannel = Channel::new();

#[embassy_executor::task(pool_size = 2)]
async fn monitor_button(
    mut button: ExtiInput<'static>,
    down: Event,
    up: Event,
    events: EventSender,
) {
    loop {
        button.wait_for_rising_edge().await;
        events.send(down.clone()).await;
        button.wait_for_falling_edge().await;
        events.send(up.clone()).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut ai_button = ExtiInput::new(p.PC0, p.EXTI0, Pull::Up);
    let mut ptt_button = ExtiInput::new(p.PC1, p.EXTI1, Pull::Up);

    unwrap!(spawner.spawn(monitor_button(
        ai_button,
        Event::AiDown,
        Event::AiUp,
        EVENT_QUEUE.sender()
    )));

    unwrap!(spawner.spawn(monitor_button(
        ptt_button,
        Event::PttDown,
        Event::PttUp,
        EVENT_QUEUE.sender()
    )));

    loop {
        let event = EVENT_QUEUE.receive().await;

        match event {
            Event::AiDown => {
                info!("AI down")
            }
            Event::AiUp => {
                info!("AI up")
            }
            Event::PttDown => {
                info!("PTT down")
            }
            Event::PttUp => {
                info!("PTT up")
            }
        }
    }
}

/*
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // Configure LEDs
    let led_r = Output::new(p.PA4, Level::Low, Speed::Low);
    let led_g = Output::new(p.PC5, Level::Low, Speed::Low);
    let led_b = Output::new(p.PB3, Level::Low, Speed::Low);

    let mut leds = [led_r, led_g, led_b];

    // Configure MGMT UART
    let config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityEven;
        config
    };
    let mut usart = Uart::new(
        p.USART1, p.PA10, p.PA9, Irqs, p.DMA2_CH7, p.DMA2_CH5, config,
    )
    .unwrap();

    unwrap!(usart.blocking_write(b"Hello Embassy World!\r\n"));
    info!("wrote Hello, starting echo");

    // let mut buf = [0u8; 4];

    for i in 0.. {
        // XXX(RLB) This seems like it ought to work, but it's not clear that the MGMT chip is
        // actually forwarding input from the USB side to the UI chip side.
        // unwrap!(usart.blocking_read(&mut buf));
        // info!("read blocking");

        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", i).unwrap();

        unwrap!(usart.write(s.as_bytes()).await);
        info!("wrote DMA");

        let led = &mut leds[i % leds.len()];

        info!("high");
        led.set_low();
        Timer::after_millis(1000).await;

        info!("low");
        led.set_high();
        Timer::after_millis(1000).await;
    }
}
*/
