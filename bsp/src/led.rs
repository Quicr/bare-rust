//! # LED Module
//!
//! This module provides functionality for controlling the LED on the board.
//! It includes methods for initializing the LED and setting its colors.
//!
//! ## Enums
//!
//! - `Color`: An enumeration of possible LED colors (Black, White, Red, Green, Blue).
//!
//! ## Functions
//!
//! - `set`: Sets the color of the LEDs based on the provided `Color` enum value.
//!
//! ## Example
//!
//! ```rust
//!  use bsp::BSP;
//!  use bsp::led;
//!  let mut bsp = BSP::new();
//!  bsp.init();
//!
//!  led::set(led::Color::Blue);
//!  led::set(led::Color::Black); // turn off LED
//! ```
//!

extern crate hal;

use crate::board;

#[cfg(feature = "std")]
extern crate std;

pub struct Led {}

impl Led {
    #[inline(never)]
    pub fn new() -> Self {
        Led {}
    }

    #[inline(never)]
    pub fn init(&self) {
        board::info::LED_RED_PIN.output();
        board::info::LED_GREEN_PIN.output();
        board::info::LED_BLUE_PIN.output();

        set(Color::Black);
    }
}

/// Represents the possible colors for the LED.
pub enum Color {
    Black, // probably should not use
    White,
    Red, // only used for errors
    Green,
    Blue,
    Teal,
    Yellow,
    Purple,
}

/// Sets the color of the LED based on the provided `Color` enum value.
pub fn set(c: Color) {
    let (mut r, mut g, mut b) = match c {
        Color::Black => (0, 0, 0),
        Color::White => (255, 255, 255),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Teal => (0, 255, 255),
        Color::Yellow => (255, 255, 0),
        Color::Purple => (255, 0, 255),
    };

    let invert_leds = board::info::LEDS_INVERTED;
    if invert_leds {
        r = 100 - r;
        g = 100 - g;
        b = 100 - b;
    }

    if r > 50 {
        board::info::LED_RED_PIN.high();
    } else {
        board::info::LED_RED_PIN.low();
    }
    if g > 50 {
        board::info::LED_GREEN_PIN.high();
    } else {
        board::info::LED_GREEN_PIN.low();
    }
    if b > 50 {
        board::info::LED_BLUE_PIN.high();
    } else {
        board::info::LED_BLUE_PIN.low();
    }

    #[cfg(feature = "std")]
    std::print!("LED: {},{},{}\r\n", r, g, b);
}
