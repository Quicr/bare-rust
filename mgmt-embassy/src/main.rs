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
    commands::{CommandContext, TlvParser},
    gpio::{GpioPeripherals, NetControl, RgbLed, UiControl},
    state::{State, DEFAULT_STATE},
    uart::{UartRouting, DMA_BUFFER_SIZE},
};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3_4 => usart::InterruptHandler<peripherals::USART3>;
});

// Static allocations for state and chip controls
static UART_ROUTING: Mutex<ThreadModeRawMutex, UartRouting> = Mutex::new(UartRouting::new());
static STATE: Mutex<ThreadModeRawMutex, State> = Mutex::new(DEFAULT_STATE);
static UI_CONTROL: Mutex<ThreadModeRawMutex, Option<UiControl>> = Mutex::new(None);
static NET_CONTROL: Mutex<ThreadModeRawMutex, Option<NetControl>> = Mutex::new(None);

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
    gpio.ui_control.normal_mode().await;
    gpio.net_control.normal_mode().await;

    // Set default logging based on initial state
    {
        let state = STATE.lock().await;
        let mut routing = UART_ROUTING.lock().await;
        match *state {
            State::Debug => {
                routing.ui_path = ui_embassy::uart::TxPath::Usb;
                routing.net_path = ui_embassy::uart::TxPath::Usb;
            }
            _ => {}
        }
    }

    // Store chip controls in statics for command context
    *UI_CONTROL.lock().await = Some(gpio.ui_control);
    *NET_CONTROL.lock().await = Some(gpio.net_control);

    // Create command context
    let context = CommandContext {
        routing: &UART_ROUTING,
        ui_control: unsafe { core::mem::transmute(&UI_CONTROL) },
        net_control: unsafe { core::mem::transmute(&NET_CONTROL) },
        state: &STATE,
    };

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

        // Read from USB UART and route
        match usb_rx.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let routing = UART_ROUTING.lock().await;
                route_data(
                    &buf[..n],
                    routing.usb_path,
                    &mut usb_tx,
                    &mut ui_tx,
                    &mut net_tx,
                )
                .await;

                // Parse commands from internal
                if routing.usb_path == ui_embassy::uart::TxPath::Internal {
                    drop(routing);
                    if let Some((command, cmd_data)) = parser.process(&buf[..n]) {
                        info!("Received command: {:?}", command);
                        if let Some(response) = context.execute(command, &cmd_data).await {
                            let _ = usb_tx.write(response).await;
                        }
                    }
                }
            }
            _ => {}
        }

        // Read from UI UART and route
        match ui_rx.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let routing = UART_ROUTING.lock().await;
                route_data(
                    &buf[..n],
                    routing.ui_path,
                    &mut usb_tx,
                    &mut ui_tx,
                    &mut net_tx,
                )
                .await;
            }
            _ => {}
        }

        // Read from NET UART and route
        match net_rx.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let routing = UART_ROUTING.lock().await;
                route_data(
                    &buf[..n],
                    routing.net_path,
                    &mut usb_tx,
                    &mut ui_tx,
                    &mut net_tx,
                )
                .await;
            }
            _ => {}
        }

        embassy_time::Timer::after(embassy_time::Duration::from_micros(100)).await;
    }
}

// Helper function to route data between UARTs
async fn route_data(
    data: &[u8],
    path: ui_embassy::uart::TxPath,
    usb_tx: &mut embassy_stm32::usart::UartTx<'static, Async>,
    ui_tx: &mut embassy_stm32::usart::UartTx<'static, Async>,
    net_tx: &mut embassy_stm32::usart::UartTx<'static, Async>,
) {
    use ui_embassy::uart::TxPath;

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
        TxPath::UiNet => {
            let _ = ui_tx.write(data).await;
            let _ = net_tx.write(data).await;
        }
        TxPath::Internal => {
            // Command parsing happens inline in main loop
        }
    }
}
