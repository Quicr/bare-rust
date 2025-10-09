#![no_std]
#![no_main]

mod board;

use board::{Board, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::{App, Event};

use cortex_m::singleton;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{mode::Async, usart::UartRx};
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
async fn monitor_net(from: UartRx<'static, Async>, events: EventSender) {
    const DMA_BUFFER_SIZE: usize = 1024;

    // Wrap the raw receiver in a DMA-buffered, SLIP-parsing, TLV-parsing version
    let mut dma_buf = [0u8; DMA_BUFFER_SIZE];
    let mut from = from.into_ring_buffered(&mut dma_buf);
    let mut from = NetRx::new(&mut from);

    loop {
        let from_net = from.next().await;
        events.send(Event::FromNet(from_net)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("about to instantiate board");

    const BUFFER_SIZE: usize = 2 * ui_app::FRAME_SIZE;

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

    /*
    board.audio_control().start();

    let mut i2s = board.i2s.take().unwrap();
    i2s.start();

    let mut frames = FrameAudio { i2s };

    let square_wave = {
        const AMPLITUDE: u16 = 0x1fff;
        const LAMBDA: usize = 18;
        let hi = core::iter::repeat(AMPLITUDE).take(LAMBDA);
        let lo = core::iter::repeat(0).take(LAMBDA);

        hi.chain(lo).cycle().take(CHANNELS * SAMPLES_PER_SEC)
    };

    trace!("before tx");
    frames.write_iter(square_wave).await;
    trace!("after tx");

    const RECORDING_LENGTH: usize = 2;
    trace!("recording {} seconds audio", RECORDING_LENGTH);
    let mut recording = [0u16; RECORDING_LENGTH * CHANNELS * SAMPLES_PER_SEC];
    for chunk in recording.chunks_mut(FRAME_SIZE) {
        let frame = frames.read().await;
        chunk.copy_from_slice(&frame.0);

        trace!("rec {}", frame.0.iter().map(|x| *x as usize).sum::<usize>());
    }

    trace!("playing recording");
    for chunk in recording.chunks(FRAME_SIZE) {
        let mut frame = Frame::zero();
        frame.0.copy_from_slice(chunk);
        frames.write(&frame).await;

        trace!(
            "play {}",
            frame.0.iter().map(|x| *x as usize).sum::<usize>()
        );
    }
    */

    // Main event loop
    let receiver = EventSource(EVENT_QUEUE.receiver());
    app.run(receiver, board).await;

    // here
    // app.handle(EVENT_QUEUE, board).await

    // in ui_ap
    // async fn handle(EVENT_QUEUE, board) {
    //      loop match event {
    //          ButtonDown => record(EVENT_QUEUE, &mut board),
    //          StartOfTalk => play(EVENT_QUEUE, &mut board),
    //      }
    // }
    //
    // async fn record(...) {
    //      loop select {
    //          event = EVENT_QUEUE.receive() => match event {
    //              ButtonUp => return;
    //              _ => self.handle_event(event)
    //          }
    //          frame = board.audio().read() => {
    //              // Transmit frame
    //          }
    //      }
    // }
    //
    // async fn play(...) {
    //      loop match event {
    //          ReceivedFrame => board.audio().write(frame),
    //          EndOfTalk => return;
    //      }
    // }
}
