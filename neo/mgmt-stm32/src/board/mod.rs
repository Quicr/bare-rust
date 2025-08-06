pub mod led;

use crate::hal::uart::{USART1, USART2};
use crate::hal::{clock, gpio, uart};
use led::Led;

use crate::hal::gpio::Fields as _;
use crate::hal::uart::Fields as _;

mod led_a {
    use crate::hal::gpio::gpioa;
    pub type RedPin = gpioa::PA4;
    pub type GreenPin = gpioa::PA6;
    pub type BluePin = gpioa::PA7;
}

mod led_b {
    use crate::hal::gpio::gpiob;
    pub type RedPin = gpiob::PB0;
    pub type GreenPin = gpiob::PB14;
    pub type BluePin = gpiob::PB15;
}

mod console {
    use crate::hal::{gpio::gpioa, uart};
    pub type Tx = gpioa::PA9;
    pub type Rx = gpioa::PA10;
    pub type Usart = uart::USART1;
    pub const BAUD_RATE: u32 = 115_200;
}

mod ui {
    use crate::hal::gpio::{gpioa, gpiob};
    use crate::hal::uart;

    pub type Tx = gpioa::PA2;
    pub type Rx = gpioa::PA3;
    pub type Usart = uart::USART2;
    pub const BAUD_RATE: u32 = 115_200;

    pub type BootPin = gpioa::PA15;
    pub type ResetPin = gpiob::PB3;
}

pub struct Board {
    pub led_a: Led<led_a::RedPin, led_a::GreenPin, led_a::BluePin, true>,
    pub led_b: Led<led_b::RedPin, led_b::GreenPin, led_b::BluePin, true>,

    pub ui_boot: gpio::Output<ui::BootPin>,
    pub ui_reset: gpio::Output<ui::ResetPin>,

    pub console: uart::Uart<console::Tx, console::Rx, console::Usart>,
    pub ui_uart: uart::Uart<ui::Tx, ui::Rx, ui::Usart>,
}

impl Board {
    pub fn new(hse_clk_freq: u32) -> Self {
        clock::init(hse_clk_freq);
        gpio::init();
        uart::init();

        let ui_boot = ui::BootPin::output();
        ui_boot.set_low();

        let ui_reset = ui::ResetPin::output();
        ui_reset.open_drain();

        Self {
            led_a: Default::default(),
            led_b: Default::default(),

            ui_boot,
            ui_reset,

            console: USART1::get(console::BAUD_RATE),
            ui_uart: USART2::get(ui::BAUD_RATE),
        }
    }
}
