#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use defmt::*;
use embassy_executor::Spawner;
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
            defmt::info!("kbd event: {:?}", event);
            events.send(event).await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut board = Board::new().await;
    let mut app = App::new();

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

    app.start(&mut board);

    // Main event loop
    loop {
        let event = EVENT_QUEUE.receive().await;
        app.handle(event, &mut board);
    }
}
