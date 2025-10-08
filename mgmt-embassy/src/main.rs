#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    mode::Async,
    peripherals, usart,
    usart::{Config, DataBits, Parity, StopBits, Uart, UartRx, UartTx},
};
use {defmt_rtt as _, panic_probe as _};

use ui_embassy::gpio::{GpioPeripherals, NetControl, RgbLed, UiControl};
use ui_embassy::state::State;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    // TODO: USART3/4 may not be fully supported by Embassy on STM32F072CB
    // Need to investigate correct UART peripheral for NET (PB10/PB11)
});

type Tx = UartTx<'static, Async>;
type Rx = UartRx<'static, Async>;

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

    // TODO: Configure UART3/4 (NET) - need to figure out correct peripheral for PB10/PB11
    // For STM32F072CB, USART3/4 might have limited Embassy support
    // let net_config = ...
    // let net_uart = ...

    info!("UARTs initialized (USB, UI). NET UART TODO");

    // TODO: Set up UART routing and command handling
    // For now, just blink LEDs to show we're alive
    loop {
        gpio.led_a.toggle_green();
        gpio.led_b.toggle_blue();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }
}
