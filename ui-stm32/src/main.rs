#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard};
use ui_app::{App, Event, Screen};

use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use {defmt_rtt as _, panic_probe as _};

const EVENT_QUEUE_DEPTH: usize = 10;
type EventChannel = Channel<CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;
type EventSender = Sender<'static, CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;

static EVENT_QUEUE: EventChannel = Channel::new();

#[embassy_executor::task(pool_size = 2)]
async fn monitor_button(mut button: Button, down: Event, up: Event, events: EventSender) {
    loop {
        button.wait_for_rising_edge().await;
        events.send(down).await;
        button.wait_for_falling_edge().await;
        events.send(up).await;
    }
}

#[embassy_executor::task]
async fn monitor_keyboard(mut keyboard: Keyboard, events: EventSender) {
    loop {
        for event in keyboard.scan() {
            events.send(event).await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut board = Board::new();
    let mut app = App::new();

    // Capture button events
    unwrap!(spawner.spawn(monitor_button(
        board.ai_button.take().unwrap(),
        Event::AiDown,
        Event::AiUp,
        EVENT_QUEUE.sender()
    )));

    unwrap!(spawner.spawn(monitor_button(
        board.ptt_button.take().unwrap(),
        Event::PttDown,
        Event::PttUp,
        EVENT_QUEUE.sender()
    )));

    // Capture keyboard events
    unwrap!(spawner.spawn(monitor_keyboard(
        board.keyboard.take().unwrap(),
        EVENT_QUEUE.sender()
    )));

    board.screen.init().await; // XXX
    board.screen.fill(0b00000_111111_00000); // XXX

    app.start(&mut board);

    // Main event loop
    loop {
        let event = EVENT_QUEUE.receive().await;
        app.handle(event, &mut board);
    }
}
