//! # GPIO Module
//!
//! This module provides functionality for initializing and controlling
//! General-Purpose Input/Output (GPIO) pins.
//! It includes methods for setting pins as input or output, configuring
//! pull-up or pull-down resistors, and reading or writing pin states.
//!
//! ## Structures
//!
//! - `Pin`: Represents a GPIO pin and provides methods to configure and control it.
//!
//! ## Functions
//!
//! - `init`: Initializes the GPIO peripheral by enabling the necessary clocks.
//!
//! ## Methods for `Pin`
//!
//! - `new`: Creates a new `Pin` instance.
//! - `output`: Configures the pin as an output.
//! - `input`: Configures the pin as an input with a pull-down resistor.
//! - `pulldown`: Configures the pin with a pull-down resistor.
//! - `pullup`: Configures the pin with a pull-up resistor.
//! - `low`: Sets the pin state to low.
//! - `high`: Sets the pin state to high.
//! - `read`: Reads the current state of the pin.
//!
//! ## Usage
//!
//! This module is intended for low-level hardware interaction and should be used with caution.
//! It provides direct access to hardware registers,
//! which can lead to undefined behavior if used incorrectly.
//!
//! ## Example
//!
//! ```rust
//! use crate::hal::gpio::{self, Pin};
//! use crate::hal::cpu;
//! use crate::hal::clock;
//!
//! fn main() {
//!     // Initialize things
//!     cpu::init();
//!     clock::init( 16_000_000 );
//!     gpio::init();
//!
//!     // Create a new pin instance
//!     let pin = Pin::new(cpu::GPIOA, 5);
//!
//!     // Configure the pin as output and set it high
//!     pin.output();
//!     pin.high();
//!
//!     // Read the pin state
//!     let state = pin.read();
//!     println!("Pin state: {}", state);
//! }
//! ```

use core::ptr;

use super::cpu;
pub use super::cpu::*;
//pub use super::svd::*;

#[cfg(feature = "stm32f405")]
pub fn init() {
    cpu::write!(RCC.ahb1enr[GPIOAEN;1], 1 );
    cpu::write!(RCC.ahb1enr[GPIOBEN;1], 1 );
    cpu::write!(RCC.ahb1enr[GPIOCEN;1], 1 );
}

#[cfg(feature = "stm32f072")]
pub fn init() {
    cpu::write!(RCC.ahbenr[IOPAEN;1], 1);
    cpu::write!(RCC.ahbenr[IOPBEN;1], 1);
}

#[derive(Copy, Clone)]
pub struct Pin(pub *mut cpu::GpioReg, pub u8);

impl Pin {
    #[allow(dead_code)]
    pub fn new(gpio: *mut cpu::GpioReg, p: u8) -> Pin {
        assert!(p < 16);
        return Pin(gpio, p);
    }

    #[inline(never)]
    pub fn output(&self) {
        let gpio = self.0;
        let pin_num = self.1;

        debug_assert!(pin_num < 16);

        // set mode to output
        cpu::write!( gpio.moder[pin_num*2;2], 0b01);

        // set output as low
        cpu::write!( gpio.odr[pin_num*1;1], 0b0);

        // set as push-pull
        cpu::write!( gpio.otyper[pin_num*1;1], 0b0);

        // set no pull up , no pull down
        cpu::write!( gpio.pupdr[pin_num*2;2], 0b00);

        // set speed to slow
        cpu::write!( gpio.ospeedr[pin_num*2;2], 0b00);
    }

    #[inline(never)]
    pub fn alt_fun(&self, af_mode: u8, fast: bool) {
        let gpio = self.0;
        let pin_num = self.1;

        debug_assert!(pin_num < 16);
        debug_assert!(af_mode < 16);

        // set mode to output
        cpu::write!( gpio.moder[pin_num*2;2], 0b10);

        // set output as low
        cpu::write!( gpio.odr[pin_num*1;1], 0b0);

        // set as push-pull
        cpu::write!( gpio.otyper[pin_num*1;1], 0b0);

        // set no pull up , no pull down
        cpu::write!( gpio.pupdr[pin_num*2;2], 0b00);

        if fast {
            // set speed to fast
            cpu::write!( gpio.ospeedr[pin_num*2;2], 0b10);
        } else {
            // set speed to slow
            cpu::write!( gpio.ospeedr[pin_num*2;2], 0b00);
        }

        if pin_num < 8 {
            cpu::write!( gpio.afrl[(pin_num)*4;4], af_mode as u32 );
        } else {
            cpu::write!( gpio.afrh[(pin_num-8)*4;4], af_mode as u32 );
        }
    }

    #[inline(never)]
    pub fn open_drain(&self) {
        let gpio = self.0;
        let pin_num = self.1;

        // set mode to output
        cpu::write!( gpio.moder[pin_num*2;2], 0b01);

        // Set output type to open-drain
        cpu::write!(gpio.otyper[pin_num * 1; 1], 0b1);

        // set output as high
        cpu::write!( gpio.odr[pin_num*1;1], 0b0);

        // set  pull up
        cpu::write!( gpio.pupdr[pin_num*2;2], 0b01);

        // set speed to slow
        cpu::write!( gpio.ospeedr[pin_num*2;2], 0b00);
    }

    #[inline(never)]
    pub fn input(&self) {
        let gpio = self.0;
        let pin_num = self.1;

        // set to pull down
        cpu::write!( gpio.pupdr[pin_num*2;2], 0b10);

        // set mode to input
        cpu::write!( gpio.moder[pin_num*2;2], 0b00);
    }

    #[inline(never)]
    #[allow(dead_code)]
    pub fn pulldown(&self) {
        let gpio = self.0;
        let pin_num = self.1;

        // set to pull down
        cpu::write!(gpio.pupdr[pin_num * 2; 2], 0b10);
    }

    #[inline(never)]
    #[allow(dead_code)]
    pub fn pullup(&self) {
        let gpio = self.0;
        let pin_num = self.1;

        // set to pull up
        cpu::write!(gpio.pupdr[pin_num * 2; 2], 0b01);
    }

    #[inline(always)]
    pub fn low(&self) {
        let gpio = self.0;
        let pin_num = self.1;

        cpu::write!(gpio.bsrr, 0b1 << (pin_num + 16));
    }

    #[inline(always)]
    pub fn high(&self) {
        let gpio = self.0;
        let pin_num = self.1 as u32;

        cpu::write!(gpio.bsrr, 0b1 << (pin_num + 0));
    }

    #[inline(always)]
    pub fn read(&self) -> bool {
        let gpio = self.0;
        let pin_num = self.1;

        let val = cpu::read!( gpio.idr[pin_num*1;1] );
        val != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_output() {
        let gpio = 0x40020000 as *mut cpu::GpioReg; // Example GPIO base address
        let pin = Pin::new(gpio, 5);
        pin.output();

        // Verify that the pin is set to output mode
        assert_eq!(cpu::read!(gpio.moder[5 * 2; 2]), 0b00);

        // Verify that the pin is set to low
        assert_eq!(cpu::read!(gpio.odr[5 * 1; 1]), 0b0);
    }

    #[test]
    fn test_pin_input() {
        let gpio = 0x40020000 as *mut cpu::GpioReg; // Example GPIO base address
        let pin = Pin::new(gpio, 5);
        pin.input();

        // Verify that the pin is set to input mode
        assert_eq!(cpu::read!(gpio.moder[5 * 2; 2]), 0b00);
    }

    #[test]
    fn test_pin_read() {
        let gpio = 0x40020000 as *mut cpu::GpioReg; // Example GPIO base address
        let pin = Pin::new(gpio, 5);

        // Simulate pin state
        cpu::write!(gpio.idr[5 * 1; 1], 1);
        assert_eq!(pin.read(), false);

        cpu::write!(gpio.idr[5 * 1; 1], 0);
        assert_eq!(!pin.read(), true);
    }
}
