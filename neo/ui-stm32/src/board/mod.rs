mod button;
mod led;

use crate::hal::gpio::{gpioa, gpioc};
use button::Button;
use led::Led;

type RedLedPin = gpioa::PA6;
type GreenLedPin = gpioc::PC5;
type BlueLedPin = gpioa::PA1;

pub struct Board {
    status_led: Led<gpioa::PA6, gpioc::PC5, gpioa::PA1>,
    ptt_button: Button<gpioc::PC0>,
    ai_button: Button<gpioc::PC1>,
}
