#![no_std]
#![no_main]

mod commands;
mod drivers;
mod state;

use defmt::*;
use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use state::{Interface, State};

pub const READ_BUFFER_SIZE: usize = 64;
pub const DMA_BUFFER_SIZE: usize = 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut usb_rx_buf = [0u8; DMA_BUFFER_SIZE];
    let mut ui_rx_buf = [0u8; DMA_BUFFER_SIZE];
    let mut net_rx_buf = [0u8; DMA_BUFFER_SIZE];

    let mut s = State::new(&mut usb_rx_buf, &mut ui_rx_buf, &mut net_rx_buf);

    info!("Starting main loop");

    let mut buf = [0u8; 64];
    let mut led_timer = embassy_time::Instant::now();

    // Main loop - handle UART routing and command parsing
    loop {
        // Blink LED A every second
        if led_timer.elapsed().as_millis() >= 1000 {
            s.led_a.toggle_green();
            led_timer = embassy_time::Instant::now();
        }

        // USB interface is special because it might be routed or read for commands
        if s.routing.usb == Interface::Command {
            s.handle_command(&mut buf).await;
        } else {
            s.route_data(Interface::Usb, &mut buf).await;
        }

        // Read and route data from the other interfaces
        s.route_data(Interface::Ui, &mut buf).await;
        s.route_data(Interface::Net, &mut buf).await;

        embassy_time::Timer::after(embassy_time::Duration::from_micros(100)).await;
    }
}
