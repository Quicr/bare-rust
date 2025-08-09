#![no_std]
#![no_main]

mod app;
mod board;

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f0xx_hal::prelude::*;

use app::{App, Event};
use board::led::Color;
use board::{Board, Timer1Hz};

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use heapless::mpmc::Q64;
use stm32f0xx_hal::pac::interrupt;

static GINT: Mutex<RefCell<Option<Timer1Hz>>> = Mutex::new(RefCell::new(None));
static EVENT_QUEUE: Q64<Event> = Q64::new();

#[interrupt]
fn TIM7() {
    static mut INT: Option<Timer1Hz> = None;

    let int = INT.get_or_insert_with(|| {
        cortex_m::interrupt::free(|cs| GINT.borrow(cs).replace(None).unwrap())
    });

    // XXX(RLB) Silently drops events
    let _ = EVENT_QUEUE.enqueue(Event::Timer1Hz);
    int.wait().ok();
}

#[entry]
fn main() -> ! {
    // Instantiate the board
    let mut board = Board::new();

    // Instantiate the app
    let mut app = App::default();

    // Configure the LEDs
    board.led_a.set(Color::Blue);
    board.led_b.set(Color::Red);

    // Make shared resources available to interrupts
    cortex_m::interrupt::free(move |cs| {
        *GINT.borrow(cs).borrow_mut() = Some(board.timer);
    });

    // Working buffer for the line we are building
    let (mut tx, mut rx) = board.console.split();

    loop {
        // Poll inputs that don't come in asynchronously
        if let Ok(byte) = rx.read() {
            // XXX(RLB) Silently drops events
            let _ = EVENT_QUEUE.enqueue(Event::ConsoleRx(byte));
        }

        // Process the next event
        if let Some(event) = EVENT_QUEUE.dequeue() {
            app.handle(event, &mut tx, &mut board.led_a);
        }
    }
}
