#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{mode::Async, usart::UartRx};
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
            defmt::info!("kbd event: {:?}", event);
            events.send(event).await;
        }
    }
}

#[embassy_executor::task]
async fn monitor_uart(mut from: UartRx<'static, Async>) {
    // const DMA_BUFFER_SIZE: usize = 1024;

    // Configure a ring buffer on the DMA receiver
    // let mut dma_buf = [0u8; DMA_BUFFER_SIZE];
    // let mut from = from.into_ring_buffered(&mut dma_buf);

    // Log results
    let mut buf = [0u8; 128];
    loop {
        let n = unwrap!(from.read_until_idle(&mut buf).await);
        let hex: heapless::String<256> = (&buf[..n]).encode_hex();
        defmt::info!("net rx: [{}]", hex);
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("about to instantiate board");

    let mut board = Board::new().await;
    let mut app = App::new();

    info!("done setting up board and app");

    // Capture button events
    unwrap!(spawner.spawn(monitor_button(
        board.ai_button.take().unwrap(),
        ButtonId::A,
        EVENT_QUEUE.sender()
    )));

    unwrap!(spawner.spawn(monitor_button(
        board.ptt_button.take().unwrap(),
        ButtonId::B,
        EVENT_QUEUE.sender()
    )));

    // Capture keyboard events
    unwrap!(spawner.spawn(monitor_keyboard(
        board.keyboard.take().unwrap(),
        EVENT_QUEUE.sender()
    )));

    // Capture UART events from the NET chip
    unwrap!(spawner.spawn(monitor_uart(board.net_rx.take().unwrap())));

    // Send a Ping packet
    const PING: u8 = 0x0e;
    const PACKET: &[u8] = &[PING, 0x00, 0x00, 0x00, 0x00, 0xC0];
    loop {
        info!("ping");
        Timer::after_millis(1000).await;
        unwrap!(board.net_tx.write(PACKET).await);
    }

    /*
    debug!("app start");
    app.start(&mut board);

    // Main event loop
    loop {
        let event = EVENT_QUEUE.receive().await;
        app.handle(event, &mut board);
    }
    */
}
