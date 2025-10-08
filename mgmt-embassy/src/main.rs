#![no_std]
#![no_main]
#![allow(unused_variables)]

use core::arch::asm;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    mode::Async,
    peripherals, usart,
    usart::{Config, DataBits, Parity, StopBits, Uart, UartRx, UartTx},
};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

type Tx = UartTx<'static, Async>;
type Rx = UartRx<'static, Async>;
type Led = Output<'static>;

const DMA_BUFFER_SIZE: usize = 1024;

#[embassy_executor::task(pool_size = 2)]
async fn pipe(mut to: Tx, from: Rx, mut led: Led) {
    // Configure a ring buffer on the DMA receiver
    let mut dma_buf = [0u8; DMA_BUFFER_SIZE];
    let mut from = from.into_ring_buffered(&mut dma_buf);

    // Copy from input to output
    let mut buf = [0u8; 4];
    loop {
        let n = unwrap!(from.read(&mut buf).await);
        unwrap!(to.write(&buf[..n]).await);
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Instantiate LEDs
    let mut led_a_r = Output::new(p.PA4, Level::High, Speed::Low);
    let mut led_a_g = Output::new(p.PA6, Level::High, Speed::Low);
    let mut led_a_b = Output::new(p.PA7, Level::High, Speed::Low);
    let mut led_b_r = Output::new(p.PB0, Level::High, Speed::Low);
    let mut led_b_g = Output::new(p.PB6, Level::High, Speed::Low);
    let mut led_b_b = Output::new(p.PB15, Level::High, Speed::Low);

    // Grab the UI Boot and Reset pins
    let ui_nrst = Output::new(p.PB3, Level::High, Speed::Low);

    led_b_r.set_low();
    led_b_b.set_low();
    led_a_r.set_high();
    led_a_g.set_low();

    // Configure USB-side UART
    let config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityNone;
        config
    };
    let usb_uart = Uart::new(
        p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH2, p.DMA1_CH3, config,
    )
    .unwrap();

    let (usb_tx, usb_rx) = usb_uart.split();

    // Echo the USB UART back to itself
    unwrap!(spawner.spawn(pipe(usb_tx, usb_rx, led_b_g)));
}
