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
        // XXX(RLB) bare-rust writes directly to the register as a u32, but this seems like it
        // results in the same thing.
        let div = Self::APB_FREQ / baud_rate;
        Self::DIV_MANTISSA::write((div >> 8) as u16);
        Self::DIV_FRACTION::write((div & 0xff) as u8);

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

    pub fn write_byte(&self, byte: u8) {
        while !F::TXE::read() {}
        F::TDR::write(byte.into())
    }

    pub fn read_byte(&self) -> u8 {
        while !F::RXNE::read() {}
        F::RDR::read() as u8
    }

    pub fn write(&self, buffer: impl AsRef<[u8]>) {
        for byte in buffer.as_ref() {
            self.write_byte(*byte);
        }
    }

    pub fn read_exact(&self, mut buffer: impl AsMut<[u8]>) {
        buffer.as_mut().fill_with(|| self.read_byte());
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
