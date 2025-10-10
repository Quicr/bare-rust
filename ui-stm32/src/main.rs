#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{mode::Async, usart::RingBufferedUartRx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Configuration parameters
const EVENT_QUEUE_DEPTH: usize = 10;
const KEYBOARD_SCAN_MILLIS: u64 = 50;

type EventChannel = Channel<CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;
type EventSender = Sender<'static, CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;
type EventReceiver = Receiver<'static, CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;

static EVENT_QUEUE: EventChannel = Channel::new();

struct EventSource(EventReceiver);

impl ui_app::EventSource for EventSource {
    async fn receive(&mut self) -> Option<Event> {
        Some(self.0.receive().await)
    }
}

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
        // XXX(RLB) This is currently broken.  The Timer call blocks forever.  I think something in
        // the clock config has broken timers, but I'm not sure what.
        let _ = Timer::after_millis(KEYBOARD_SCAN_MILLIS).await;
        for event in keyboard.scan() {
            defmt::trace!("keyboard event {}", event);
            events.send(event).await;
        }
    }
}

#[embassy_executor::task]
async fn monitor_net(mut from: RingBufferedUartRx<'static>, events: EventSender) {
    let mut from = NetRx::new(&mut from);

    loop {
        let from_net = from.next().await;
        events.send(Event::FromNet(from_net)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("about to instantiate board");

    const NET_RX_BUFFER_SIZE: usize = core::mem::size_of::<ui_app::FromNet>();
    const I2S_BUFFER_SIZE: usize = 2 * ui_app::FRAME_SIZE;

    let net_rx_buf = singleton!(: [u8; NET_RX_BUFFER_SIZE] = [0; NET_RX_BUFFER_SIZE]).unwrap();
    let i2s_tx = singleton!(: [u16; I2S_BUFFER_SIZE] = [0; I2S_BUFFER_SIZE]).unwrap();
    let i2s_rx = singleton!(: [u16; I2S_BUFFER_SIZE] = [0; I2S_BUFFER_SIZE]).unwrap();

    let mut board = Board::new(net_rx_buf, i2s_tx, i2s_rx);
    let mut app = App::new();

    info!("done setting up board and app");

    // Capture button events
    spawner.spawn(
        monitor_button(
            board.button_a.take().unwrap(),
            ButtonId::A,
            EVENT_QUEUE.sender(),
        )
        .unwrap(),
    );

    spawner.spawn(
        monitor_button(
            board.button_b.take().unwrap(),
            ButtonId::B,
            EVENT_QUEUE.sender(),
        )
        .unwrap(),
    );

    // Capture keyboard events
    spawner.spawn(monitor_keyboard(board.keyboard.take().unwrap(), EVENT_QUEUE.sender()).unwrap());

    // TODO Re-enable and debug NetRx
    // Capture UART events from the NET chip
    // spawner.spawn(monitor_net(board.net_rx.take().unwrap(), EVENT_QUEUE.sender()).unwrap());

    debug!("app start");
    app.start(&mut board);

    // Main event loop
    let receiver = EventSource(EVENT_QUEUE.receiver());
    app.run(receiver, board).await;
}
