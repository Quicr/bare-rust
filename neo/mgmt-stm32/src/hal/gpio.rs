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
    RCC::AHBENR::IOPAEN::write(true);
    RCC::AHBENR::IOPBEN::write(true);
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
    type AFR: Write<u8>;

    fn alt_fun(af_mode: u8, fast: bool) -> AltFun<Self> {
        Self::MODER::write(0b10); // Set mode to output
        Self::ODR::write(false); // Set output to low
        Self::OTYPER::write(false); // Set as push-pull
        Self::PUPDR::write(0b00); // Set no pull up; no pull down

        Self::OSPEEDR::write(if fast { 0b10 } else { 0b00 });
        Self::AFR::write(af_mode);

        AltFun::new()
    }

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

pub struct AltFun<F> {
    _phantom: core::marker::PhantomData<F>,
}

impl<F> AltFun<F>
where
    F: Fields,
{
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
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

    pub fn set(&self, high: bool) {
        if high {
            self.set_high()
        } else {
            self.set_low()
        }
    }

    pub fn set_low(&self) {
        F::BR::write(true);
    }

    pub fn set_high(&self) {
        F::BS::write(true);
    }

    pub fn open_drain(&self) {
        F::OTYPER::write(true); // Set output type to open-drain
        F::ODR::write(false); // Set output to low
                              // XXX(RLB) Comment says "high" in bare-rust
        F::PUPDR::write(0b01); // Set pull up
        F::OSPEEDR::write(0b00); // Set speed to slow
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

    pub fn pull_up(&self) {
        F::PUPDR::write(0b10);
    }

    pub fn pull_down(&self) {
        F::PUPDR::write(0b01);
    }

    pub fn read(&self) -> bool {
        F::IDR::read()
    }
}

use paste::paste;

macro_rules! pin {
    ($name:ident, $pin:literal, $lohi:ident) => {
        paste! {
            pub struct [<P $name $pin>];

            impl Fields for [<P $name $pin>] {
                type MODER = [<GPIO $name>]::MODER::[<MODER $pin>];
                type OTYPER = [<GPIO $name>]::OTYPER::[<OT $pin>];
                type OSPEEDR = [<GPIO $name>]::OSPEEDR::[<OSPEEDR $pin>];
                type PUPDR = [<GPIO $name>]::PUPDR::[<PUPDR $pin>];
                type IDR = [<GPIO $name>]::IDR::[<IDR $pin>];
                type ODR = [<GPIO $name>]::ODR::[<ODR $pin>];
                type BR = [<GPIO $name>]::BSRR::[<BR $pin>];
                type BS = [<GPIO $name>]::BSRR::[<BS $pin>];
                type AFR = [<GPIO $name>]::[<AFR $lohi>]::[<AFR $lohi $pin>];
            }
        }
    };
}

macro_rules! bus {
    ($name:expr) => {
        paste! {
            pub mod [<gpio $name:lower>] {
                use paste::paste;
                use super::Fields;
                use crate::svd::[<GPIO $name>];

                pin! { $name,  0, L }
                pin! { $name,  1, L }
                pin! { $name,  2, L }
                pin! { $name,  3, L }
                pin! { $name,  4, L }
                pin! { $name,  5, L }
                pin! { $name,  6, L }
                pin! { $name,  7, L }
                pin! { $name,  8, H }
                pin! { $name,  9, H }
                pin! { $name, 10, H }
                pin! { $name, 11, H }
                pin! { $name, 12, H }
                pin! { $name, 13, H }
                pin! { $name, 14, H }
                pin! { $name, 15, H }
            }
        }
    };
}

bus! { A }
bus! { B }
