#![no_std]
#![no_main]

mod commands;
mod gpio;
mod uart;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    mode::Async,
    peripherals, usart,
    usart::{Config, DataBits, Parity, RingBufferedUartRx, StopBits, Uart},
};
use {defmt_rtt as _, panic_probe as _};

use crate::{
    commands::{CommandContext, CommandResponse, TlvParser},
    gpio::{GpioPeripherals, NetControl, RgbLed, UiControl},
    uart::{UartRouting, DMA_BUFFER_SIZE, OK_BYTE, READY_BYTE},
};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3_4 => usart::InterruptHandler<peripherals::USART3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    // Configure UART3 (NET)
    let net_config = {
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityNone;
        config
    };
    let net_uart = Uart::new(
        p.USART3, p.PB11, // RX (NET_RX1_MGMT)
        p.PB10, // TX (NET_TX1_MGMT)
        Irqs, p.DMA1_CH7, p.DMA1_CH6, net_config,
    )
    .unwrap();

    info!("UARTs initialized");

    // Split UARTs into RX and TX
    let (mut usb_tx, usb_rx) = usb_uart.split();
    let (mut ui_tx, ui_rx) = ui_uart.split();
    let (mut net_tx, net_rx) = net_uart.split();

    // Create DMA buffers as local variables (owned by main task)
    let mut usb_rx_buf = [0u8; DMA_BUFFER_SIZE];
    let mut ui_rx_buf = [0u8; DMA_BUFFER_SIZE];
    let mut net_rx_buf = [0u8; DMA_BUFFER_SIZE];

    let mut usb_rx = usb_rx.into_ring_buffered(&mut usb_rx_buf);
    let mut ui_rx = ui_rx.into_ring_buffered(&mut ui_rx_buf);
    let mut net_rx = net_rx.into_ring_buffered(&mut net_rx_buf);

    info!("UARTs configured");

    // Initialize chips to normal mode
    gpio.ui_control.normal_mode();
    gpio.net_control.normal_mode();

    // Create local mutable variables for routing and chip controls
    let mut routing = UartRouting::default();
    let mut ui_control = gpio.ui_control;
    let mut net_control = gpio.net_control;

    // Set default logging (Debug mode: logs enabled)
    routing.ui_path = crate::uart::TxPath::Usb;
    routing.net_path = crate::uart::TxPath::Usb;

    let mut parser = TlvParser::new();

    info!("Starting main loop");

    let mut buf = [0u8; 64];
    let mut led_timer = embassy_time::Instant::now();

    // Main loop - handle UART routing and command parsing
    loop {
        // Blink LED every second
        if led_timer.elapsed().as_millis() >= 1000 {
            gpio.led_a.toggle_green();
            led_timer = embassy_time::Instant::now();
        }

        if routing.usb_path != crate::uart::TxPath::Internal {
            // If we are not reading commands from USB, read from USB UART and route
            let data = must_read(&mut usb_rx, &mut buf).await;
            route_data(data, routing.usb_path, &mut usb_tx, &mut ui_tx, &mut net_tx).await;
        } else {
            let data = must_read(&mut usb_rx, &mut buf).await;

            let (command, cmd_data) = parser.process(data).expect("Invalid command");

            info!("Received command: {:?}", command);
            let mut context = CommandContext {
                routing: &mut routing,
                ui_control: &mut ui_control,
                net_control: &mut net_control,
            };

            let response = context
                .execute(command, &cmd_data)
                .await
                .expect("Command failure");

            match response {
                CommandResponse::FlashUi => {
                    // Send OK byte
                    let _ = usb_tx.write(&[OK_BYTE]).await;

                    let flash_config = {
                        let mut config = Config::default();
                        config.baudrate = 115200;
                        config.data_bits = DataBits::DataBits9;
                        config.stop_bits = StopBits::STOP1;
                        config.parity = Parity::ParityEven;
                        config
                    };

                    usb_rx.set_config(&flash_config).unwrap();
                    usb_tx.set_config(&flash_config).unwrap();

                    // Delay to allow UART reconfiguration to settle
                    embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;

                    // Put UI chip into bootloader mode
                    ui_control.bootloader_mode();

                    // Send Ready byte
                    let _ = usb_tx.write(&[READY_BYTE]).await;
                    info!("UI flash mode active - bootloader ready");
                }
                CommandResponse::FlashNet => {
                    // Send OK byte
                    let _ = usb_tx.write(&[OK_BYTE]).await;

                    // Delay before entering bootloader mode
                    embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;

                    // Put NET chip into bootloader mode
                    net_control.bootloader_mode();

                    // Send Ready byte
                    let _ = usb_tx.write(&[READY_BYTE]).await;
                    info!("NET flash mode active - bootloader ready");
                }
                CommandResponse::ToUi(data) => {
                    // Forward data to UI UART
                    let _ = ui_tx.write(&data).await;
                }
                CommandResponse::ToNet(data) => {
                    // Forward data to NET UART
                    let _ = net_tx.write(&data).await;
                }
                CommandResponse::ToUsb(data) => {
                    // Forward data to USB UART
                    let _ = usb_tx.write(&data).await;
                }
            }
        }

        // Read from UI UART and route
        let data = must_read(&mut ui_rx, &mut buf).await;
        route_data(data, routing.ui_path, &mut usb_tx, &mut ui_tx, &mut net_tx).await;

        // Read from NET UART and route
        let data = must_read(&mut net_rx, &mut buf).await;
        route_data(data, routing.net_path, &mut usb_tx, &mut ui_tx, &mut net_tx).await;

        embassy_time::Timer::after(embassy_time::Duration::from_micros(100)).await;
    }
}

async fn must_read<'a>(rx: &mut RingBufferedUartRx<'_>, buf: &'a mut [u8]) -> &'a [u8] {
    let n = rx.read(buf).await.unwrap();
    &buf[..n]
}

// Helper function to route data between UARTs
async fn route_data(
    data: &[u8],
    path: crate::uart::TxPath,
    usb_tx: &mut embassy_stm32::usart::UartTx<'static, Async>,
    ui_tx: &mut embassy_stm32::usart::UartTx<'static, Async>,
    net_tx: &mut embassy_stm32::usart::UartTx<'static, Async>,
) {
    use crate::uart::TxPath;

    if data.is_empty() {
        return;
    }

    match path {
        TxPath::None => {}
        TxPath::Usb => {
            let _ = usb_tx.write(data).await;
        }
        TxPath::Ui => {
            let _ = ui_tx.write(data).await;
        }
        TxPath::Net => {
            let _ = net_tx.write(data).await;
        }
        TxPath::Internal => {
            // Command parsing happens inline in main loop
        }
    }
}
