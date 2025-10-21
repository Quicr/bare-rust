#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use cortex_m::singleton;
use embassy_executor::Spawner;
use embassy_stm32::usart::RingBufferedUartRx;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use {defmt as _, defmt_rtt as _, panic_probe as _};

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
        let Some(from_net) = from.next().await else {
            continue;
        };
        events.send(Event::FromNet(from_net)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("about to instantiate board");

    const NET_RX_BUFFER_SIZE: usize = core::mem::size_of::<ui_app::FromNet>();
    const I2S_BUFFER_SIZE: usize = 2 * ui_app::FRAME_SIZE;

    let net_rx_buf = singleton!(: [u8; NET_RX_BUFFER_SIZE] = [0; NET_RX_BUFFER_SIZE]).unwrap();
    let i2s_tx = singleton!(: [u16; I2S_BUFFER_SIZE] = [0; I2S_BUFFER_SIZE]).unwrap();
    let i2s_rx = singleton!(: [u16; I2S_BUFFER_SIZE] = [0; I2S_BUFFER_SIZE]).unwrap();

    let board = Board::new(net_rx_buf, i2s_tx, i2s_rx);

    #[cfg(feature = "tx-demo")]
    tx_demo(board).await;

    #[cfg(feature = "rx-demo")]
    rx_demo(board).await;

    #[cfg(not(any(feature = "tx-demo", feature = "rx-demo")))]
    app_main(board, spawner).await;
}

#[cfg(feature = "tx-demo")]
async fn tx_demo(mut board: Board) {
    use core::fmt::Write;
    use heapless::String;
    use hex::ToHex;
    use ui_app::{NetTx, Outputs, ToNet};

    let mut msg = String::default();
    let mut msg_hex: String<256> = String::default();

    for i in 0.. {
        msg.clear();
        let _ = write!(&mut msg, "{:03}", i);
        let msg_hex: String<256> = msg.encode_hex();
        defmt::trace!("tx: {} {}", msg, msg_hex);

        board.net_tx().write(&ToNet::Chat(msg.clone()));
        Timer::after_millis(1000).await;
    }
}

#[cfg(feature = "rx-demo")]
async fn rx_demo(mut board: Board) {
    use core::fmt::Write;
    use heapless::String;
    use ui_app::FromNet;

    let mut from = board.net_rx.take().unwrap();
    let mut from = NetRx::new(&mut from);

    loop {
        let Some(from_net) = from.next().await else {
            defmt::trace!("rx fail");
            continue;
        };

        match from_net {
            FromNet::Pong => defmt::trace!("rx pong (?)"),
            FromNet::AudioFrame(frame) => defmt::trace!("rx audio {} (?)", frame.0.len()),
            FromNet::Chat(msg) => defmt::trace!("rx chat [{}]", msg),
        }
    }
}

#[cfg(not(any(feature = "tx-demo", feature = "rx-demo")))]
async fn app_main(mut board: Board, spawner: Spawner) {
    let mut app = App::new();

    defmt::info!("stack usage after startup: {}", cortex_m_stack::usage());

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

    // Capture UART events from the NET chip
    spawner.spawn(monitor_net(board.net_rx.take().unwrap(), EVENT_QUEUE.sender()).unwrap());

    defmt::info!(
        "stack usage after spawning tasks: {}",
        cortex_m_stack::usage()
    );

    // Start up the app
    app.start(&mut board);

    defmt::info!("stack usage after app start: {}", cortex_m_stack::usage());

    // Main event loop
    let receiver = EventSource(EVENT_QUEUE.receiver());
    app.run(receiver, board).await;
}
