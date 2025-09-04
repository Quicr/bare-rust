#![no_std]
#![no_main]
#![allow(dead_code)] // TODO(RLB) Remove once things are more complete

mod board;
mod hal;
mod startup;
mod svd;

use board::led::Color;
use board::Board;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // XXX(RLB): Lazy static?
    let board = Board::new(16_000_000);

    // Set the LEDs to a startup pattern
    board.led_a.set(Color::White);
    board.led_b.set(Color::White);

    // Forward UART from console to UI
    loop {
        if board.ui_uart.read_ready() {
            let c = board.ui_uart.read_byte();
            board.console.write_byte(c);
        }

        if board.console.read_ready() {
            let c = board.console.read_byte();
            board.ui_uart.write_byte(c);
        }
    }
}
