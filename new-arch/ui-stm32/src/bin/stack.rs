#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use ui_stm32::stack;
use {defmt_rtt as _, embassy_stm32 as _, panic_probe as _};

fn large_stack() {
    let mut x = [0u8; 512];
    x.fill(0xAA);
}

fn small_stack() {
    let mut x = [0u8; 32];
    x.fill(0xAA);
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("stack measurement test!");

    /*
    let large = stack::measure(large_stack);
    info!("large: {}", large);

    let small = stack::measure(small_stack);
    info!("small: {}", small);
    */
}
