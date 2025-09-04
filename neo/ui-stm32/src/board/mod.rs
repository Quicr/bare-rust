mod button;
mod led;

use crate::hal::gpio::{gpioa, gpioc};
use button::Button;
use led::Led;

type RedLedPin = gpioa::PA6;
type GreenLedPin = gpioc::PC5;
type BlueLedPin = gpioa::PA1;
type PttButtonPin = gpioc::PC0;
type AiButtonPin = gpioc::PC1;

pub struct Board {
    status_led: Led<RedLedPin, GreenLedPin, BlueLedPin>,
    ptt_button: Button<PttButtonPin>,
    ai_button: Button<AiButtonPin>,
}
