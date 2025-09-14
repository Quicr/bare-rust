#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{i2c::Error, mode::Async, usart::UartRx};
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

    ///// Audio Chip /////

    // Skip I2C setup for now (?)

    /*
    ///// EEPROM /////

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
