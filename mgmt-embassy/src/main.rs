#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    mode::Async,
    peripherals, usart,
    usart::{Config, DataBits, Parity, StopBits, Uart},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use {defmt_rtt as _, panic_probe as _};

use ui_embassy::{
    gpio::{GpioPeripherals, NetControl, RgbLed, UiControl},
    uart::{uart_rx_task, uart_tx_task, TxChannels, UartRouting, DMA_BUFFER_SIZE},
};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    // TODO: USART3/4 may not be fully supported by Embassy on STM32F072CB
    // Need to investigate correct UART peripheral for NET (PB10/PB11)
});

// Static allocations for UART infrastructure
static TX_CHANNELS: TxChannels = TxChannels::new();
static UART_ROUTING: Mutex<ThreadModeRawMutex, UartRouting> = Mutex::new(UartRouting::new());

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting MGMT firmware");
    let p = embassy_stm32::init(Default::default());

    // Initialize GPIO peripherals
    let mut gpio = GpioPeripherals {
        led_a: RgbLed::new(p.PA4, p.PA6, p.PA7),  // NET LED
        led_b: RgbLed::new(p.PB0, p.PB6, p.PB15), // UI LED
        ui_control: UiControl::new(p.PB3, p.PA15, p.PB8),
        net_control: NetControl::new(p.PB4, p.PB5),
    };

    // Turn off all LEDs initially
    gpio.led_a.off();
    gpio.led_b.off();

    info!("GPIO initialized");

    // Configure UART1 (USB)
    let usb_config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityNone;
        config
    };
    let usb_uart = Uart::new(
        p.USART1, p.PA10, // RX
        p.PA9,  // TX
        Irqs, p.DMA1_CH2, p.DMA1_CH3, usb_config,
    )
    .unwrap();

    // Configure UART2 (UI)
    let ui_config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityNone;
        config
    };
    let ui_uart = Uart::new(
        p.USART2, p.PA3, // RX (UI_RX1_MGMT)
        p.PA2, // TX (UI_TX1_MGMT)
        Irqs, p.DMA1_CH4, p.DMA1_CH5, ui_config,
    )
    .unwrap();

    info!("UARTs initialized");

    // Split UARTs into RX and TX
    let (usb_tx, usb_rx) = usb_uart.split();
    let (ui_tx, ui_rx) = ui_uart.split();

    // Create ring-buffered RX
    static mut USB_RX_BUF: [u8; DMA_BUFFER_SIZE] = [0u8; DMA_BUFFER_SIZE];
    static mut UI_RX_BUF: [u8; DMA_BUFFER_SIZE] = [0u8; DMA_BUFFER_SIZE];

    let usb_rx = usb_rx.into_ring_buffered(unsafe { &mut USB_RX_BUF });
    let ui_rx = ui_rx.into_ring_buffered(unsafe { &mut UI_RX_BUF });

    // Spawn UART tasks
    spawner
        .spawn(usb_rx_task(usb_rx))
        .expect("Failed to spawn USB RX task");
    spawner
        .spawn(usb_tx_task(usb_tx))
        .expect("Failed to spawn USB TX task");
    spawner
        .spawn(ui_rx_task(ui_rx))
        .expect("Failed to spawn UI RX task");
    spawner
        .spawn(ui_tx_task(ui_tx))
        .expect("Failed to spawn UI TX task");

    info!("UART tasks spawned");

    // TODO: Spawn command parser task
    // TODO: Spawn NET UART tasks when peripheral is figured out

    // Main loop - for now just blink LEDs
    loop {
        gpio.led_a.toggle_green();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }
}

// UART task wrappers
#[embassy_executor::task]
async fn usb_rx_task(rx: embassy_stm32::usart::RingBufferedUartRx<'static>) {
    uart_rx_task(rx, &TX_CHANNELS, &UART_ROUTING, "USB", |r| r.usb_path).await;
}

#[embassy_executor::task]
async fn usb_tx_task(tx: embassy_stm32::usart::UartTx<'static, Async>) {
    uart_tx_task(tx, &TX_CHANNELS.usb, "USB").await;
}

#[embassy_executor::task]
async fn ui_rx_task(rx: embassy_stm32::usart::RingBufferedUartRx<'static>) {
    uart_rx_task(rx, &TX_CHANNELS, &UART_ROUTING, "UI", |r| r.ui_path).await;
}

#[embassy_executor::task]
async fn ui_tx_task(tx: embassy_stm32::usart::UartTx<'static, Async>) {
    uart_tx_task(tx, &TX_CHANNELS.ui, "UI").await;
}
