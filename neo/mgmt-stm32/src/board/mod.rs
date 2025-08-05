pub mod led;

use crate::hal;
use led::Led;

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

pub struct Board {
    pub led_a: Led<led_a::RedPin, led_a::GreenPin, led_a::BluePin, true>,
    pub led_b: Led<led_b::RedPin, led_b::GreenPin, led_b::BluePin, true>,
}

impl Board {
    pub fn new(hse_clk_freq: u32) -> Self {
        hal::clock::init(hse_clk_freq);
        hal::gpio::init();

        Self {
            led_a: Default::default(),
            led_b: Default::default(),
        }
    }
}
