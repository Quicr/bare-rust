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
//! ``` rust
//! use crate::hal::gpio::{self, Pin};
//! use crate::hal::cpu;
//! use crate::hal::clock;
//!
//! fn main() {
//!     cpu::init();
//!     clock::init( 16_000_000 );
//!     gpio::init();
//!
//!     let pin = gpio::gpioa::PA5::output();
//!     pin.set_high();
//!
//!     let pin = gpio::gpioa::PA4::input();
//!     println!("Pin state: {}", pin.read());
//! }
//! ```
#![allow(non_camel_case_types)]

use crate::svd::{Read, Write, RCC};

pub fn init() {
    RCC::AHB1ENR::GPIOAEN::write(true);
}

pub trait Fields: Sized {
    type IDR: Read<bool>;
    type MODER: Write<u8>;
    type ODR: Write<bool>;
    type OTYPER: Write<bool>;
    type PUPDR: Write<u8>;
    type OSPEEDR: Write<u8>;
    type BR: Write<bool>;
    type BS: Write<bool>;

    fn output() -> Output<Self> {
        Self::MODER::write(0b01); // mode = output
        Self::ODR::write(false); // output = low
        Self::OTYPER::write(false); // push-pull
        Self::PUPDR::write(0b00); // no pull down or pull up
        Self::OSPEEDR::write(0b00); // speed = slow

        Output::new()
    }

    fn input() -> Input<Self> {
        Self::MODER::write(0b00); // mode = input
        Self::PUPDR::write(0b10); // pull down

        Input::new()
    }
}

pub struct Output<F> {
    _phantom: core::marker::PhantomData<F>,
}

impl<F> Output<F>
where
    F: Fields,
{
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn set_low(&self) {
        F::BR::write(true);
    }

    pub fn set_high(&self) {
        F::BS::write(true);
    }
}

pub struct Input<F> {
    _phantom: core::marker::PhantomData<F>,
}

impl<F> Input<F>
where
    F: Fields,
{
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    fn read(&self) -> bool {
        F::IDR::read()
    }
}

use paste::paste;

macro_rules! pin {
    ($name:expr, $pin:literal) => {
        paste! {
            struct [<P $name $pin>];

            impl Fields for [<P $name $pin>] {
                type MODER = [<GPIO $name>]::MODER::[<MODER $pin>];
                type OTYPER = [<GPIO $name>]::OTYPER::[<OT $pin>];
                type OSPEEDR = [<GPIO $name>]::OSPEEDR::[<OSPEEDR $pin>];
                type PUPDR = [<GPIO $name>]::PUPDR::[<PUPDR $pin>];
                type IDR = [<GPIO $name>]::IDR::[<IDR $pin>];
                type ODR = [<GPIO $name>]::ODR::[<ODR $pin>];
                type BR = [<GPIO $name>]::BSRR::[<BR $pin>];
                type BS = [<GPIO $name>]::BSRR::[<BS $pin>];
            }
        }
    };
}

macro_rules! bus {
    ($name:expr) => {
        paste! {
            mod [<gpio $name:lower>] {
                use paste::paste;
                use super::Fields;
                use crate::svd::[<GPIO $name>];

                pin! { $name,  0 }
                pin! { $name,  1 }
                pin! { $name,  2 }
                pin! { $name,  3 }
                pin! { $name,  4 }
                pin! { $name,  5 }
                pin! { $name,  6 }
                pin! { $name,  7 }
                pin! { $name,  8 }
                pin! { $name,  9 }
                pin! { $name, 10 }
                pin! { $name, 11 }
                pin! { $name, 12 }
                pin! { $name, 13 }
                pin! { $name, 14 }
                pin! { $name, 15 }
            }
        }
    };
}

bus! { A }
