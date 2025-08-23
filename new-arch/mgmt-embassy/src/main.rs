#![no_std]
#![no_main]
#![allow(unused_variables)]

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

#[embassy_executor::task(pool_size = 2)]
async fn pipe(mut to: Tx, mut from: Rx, mut led: Led) {
    let mut buf = [0u8; 1];
    loop {
        unwrap!(from.read(&mut buf).await);
        unwrap!(to.write(&buf).await);
        led.toggle();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // Instantiate LEDs
    let led_a_r = Output::new(p.PA4, Level::Low, Speed::Low);
    let led_a_g = Output::new(p.PA6, Level::Low, Speed::Low);
    let led_a_b = Output::new(p.PA7, Level::Low, Speed::Low);
    let led_b_r = Output::new(p.PB0, Level::Low, Speed::Low);
    let led_b_g = Output::new(p.PB6, Level::Low, Speed::Low);
    let led_b_b = Output::new(p.PB15, Level::Low, Speed::Low);

    // Configure USB-side UART
    let config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityEven;
        config
    };
    let usb_uart = Uart::new(
        p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH2, p.DMA1_CH3, config,
    )
    .unwrap();

    let (usb_tx, usb_rx) = usb_uart.split();

    // Configure UI-side UART
    let config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityEven;
        config
    };
    let ui_uart = Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH4, p.DMA1_CH5, config).unwrap();

    let (ui_tx, ui_rx) = ui_uart.split();

    // Pipe the two UARTs together
    unwrap!(spawner.spawn(pipe(ui_tx, usb_rx, led_a_b)));
    unwrap!(spawner.spawn(pipe(usb_tx, ui_rx, led_b_g)));
}
