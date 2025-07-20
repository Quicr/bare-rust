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

    board.led_a.set(Color::Purple);
    board.led_b.set(Color::Teal);

    // TODO
    loop {}
}
