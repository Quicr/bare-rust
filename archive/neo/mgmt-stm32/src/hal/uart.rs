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

use crate::hal::gpio;
use crate::svd::{Read, Write, RCC};

pub fn init() {
    RCC::APB2ENR::USART1EN::write(true);
    RCC::APB1ENR::USART2EN::write(true);
}

pub trait Fields: Sized {
    const APB_FREQ: u32;

    type DIV_MANTISSA: Write<u16>;
    type DIV_FRACTION: Write<u8>;
    type PCE: Write<bool>;
    type PS: Write<bool>;
    type STOP: Write<u8>;
    type UE: Write<bool>;
    type TE: Write<bool>;
    type RE: Write<bool>;
    type TXE: Read<bool>;
    type TDR: Write<u16>;
    type RXNE: Read<bool>;
    type RDR: Read<u16>;

    fn get<Tx: gpio::Fields, Rx: gpio::Fields>(baud_rate: u32) -> Uart<Tx, Rx, Self> {
        // Acquire and configure the GPIO pins
        let _tx_pin = Tx::alt_fun(1, false); // AF1 work for USART1 to 3
        let _rx_pin = Rx::alt_fun(1, false); // AF1 work for USART1 to 3

        // Set the baud rate
        // XXX(RLB) The bare-rust equivalent of this code writes directly to the BRR register, but
        // the SVD file exposes only Mantissa and Fraction fields.  The Fraction field is 4 bits
        // wide, so this should be equivalent.
        const FRACTION_WIDTH: usize = 4;
        const FRACTION_MASK: u32 = (1 << FRACTION_WIDTH) - 1;

        let div = Self::APB_FREQ / baud_rate;
        Self::DIV_MANTISSA::write((div >> FRACTION_WIDTH) as u16);
        Self::DIV_FRACTION::write((div & FRACTION_MASK) as u8);

        // Set no parity
        Self::PCE::write(false); // No parity
        Self::STOP::write(0b00); // 1 stop bit

        // Enable the USART bus as transmitter and receiver
        Self::UE::write(true); // USART enable
        Self::TE::write(true); // Transmitter enable
        Self::RE::write(true); // Receiver enable

        Uart::new()
    }
}

pub struct Uart<Tx, Rx, F> {
    _phantom: core::marker::PhantomData<(Tx, Rx, F)>,
}

impl<Tx, Rx, F> Uart<Tx, Rx, F>
where
    Tx: gpio::Fields,
    Rx: gpio::Fields,
    F: Fields,
{
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn write_ready(&self) -> bool {
        !F::TXE::read()
    }

    pub fn read_ready(&self) -> bool {
        !F::RXNE::read()
    }

    pub fn write_byte(&self, byte: u8) {
        while !self.write_ready() {}
        F::TDR::write(byte.into())
    }

    pub fn read_byte(&self) -> u8 {
        while !self.read_ready() {}
        F::RDR::read() as u8
    }

    pub fn write(&self, buffer: impl AsRef<[u8]>) {
        for byte in buffer.as_ref() {
            self.write_byte(*byte);
        }
    }

    pub fn read_exact(&self, buffer: &mut [u8]) {
        buffer.fill_with(|| self.read_byte());
    }

    pub fn read_until<'a>(&self, delim: char, buffer: &'a mut [u8]) -> &'a [u8] {
        let mut utf8 = [0_u8; 1];
        delim.encode_utf8(&mut utf8);

        let mut len = 0;
        let mut done = false;
        buffer.fill_with(|| {
            if done {
                return 0;
            }

            let c = self.read_byte();
            done = c == utf8[0];
            len += 1;

            c
        });

        &buffer[..len]
    }
}

use crate::svd;
use paste::paste;

macro_rules! device {
    ($name:ident, $apb_freq:expr) => {
        paste! {
            pub struct $name;

            impl Fields for $name {
                const APB_FREQ: u32 = $apb_freq;
                type DIV_MANTISSA = svd::$name::BRR::DIV_Mantissa;
                type DIV_FRACTION = svd::$name::BRR::DIV_Fraction;
                type PCE = svd::$name::CR1::PCE;
                type PS = svd::$name::CR1::PS;
                type STOP = svd::$name::CR2::STOP;
                type UE = svd::$name::CR1::UE;
                type TE = svd::$name::CR1::TE;
                type RE = svd::$name::CR1::RE;
                type TXE = svd::$name::ISR::TXE;
                type TDR = svd::$name::TDR::TDR;
                type RXNE = svd::$name::ISR::RXNE;
                type RDR = svd::$name::RDR::RDR;
            }
        }
    };
}

device! { USART1, 48_000_000 }
device! { USART2, 48_000_000 }
