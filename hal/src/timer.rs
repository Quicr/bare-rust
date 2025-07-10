//! # Timer Module
//!
//! This module provides functionality for initializing and handling timers, specifically TIM2.
//! It includes methods for setting up the timer, handling timer interrupts, and retrieving the current time.
//!
//! ## Constants
//!
//! - `TIME_WRAP_AROUND`: The time duration (in microseconds) after which the timer wraps around (1 hour at 1 MHz).
//!
//! ## Functions
//!
//! - `init2`: Initializes the TIM2 timer with a 1 MHz prescaler and sets it to wrap around every hour.
//! - `handle_tim2_irq`: Handles the TIM2 interrupt by clearing the update interrupt flag.
//! - `current_time`: Retrieves the current time in microseconds from the TIM2 timer.
//!
//! ## Structures
//!
//! - `MicroSeconds`: Represents a time duration in microseconds and provides methods for time manipulation.
//!
//! ## Example
//!
//! ```rust
//! use crate::hal::timer::{self, MicroSeconds};
//! use crate::hal::cpu;
//!
//! fn main() {
//!     // Initialize the CPU and timer
//!     cpu::init();
//!     timer::init2();
//!
//!     // Get the current time
//!     let current_time = timer::current_time();
//! }
//! ```

#[cfg(not(feature = "stm32f072"))]
use core::ptr;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(feature = "stm32f072"))]
use super::cpu;

#[cfg(not(feature = "stm32f072"))]
use super::cpu::*;

#[cfg(feature = "stm32f405")]
pub use super::cpu::TIM_GEN as TIM2;

const TIME_WRAP_AROUND: u32 = 3600 * 1000_000; // 1 hour@1Mhz

#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn init2() {
    // enable TIM2 clock
    //cpu::write!( RCC.apb2enr[TIM2EN;1], 1);
    cpu::write!( RCC.apb1enr[TIM2EN;1], 1);

    // TIM1 is apb2 timer clock is 168MHz and 16 bits
    // TIM2 is apb1 timer clock is 84MHz and 32 bits

    // set prescaler for 1MHz
    cpu::write!(TIM2.psc, 84 - 1);

    // set auto-reload for 3600 seconds (1 hour)
    cpu::write!(TIM2.arr, TIME_WRAP_AROUND - 1);

    // force load of prescaler and auto-reload
    cpu::write!( TIM2.egr[UG;1], 1);

    // enable update interrupt
    cpu::write!( TIM2.dier[UIE;1], 1);

    // enable counter
    cpu::write!( TIM2.cr1[CEN;1], 1);

    // enable interrupt in NVIC
    const TIM2_IRQ_NUM: usize = 28;
    const IRQ_INDEX: usize = TIM2_IRQ_NUM / 32;
    const IRQ_BIT: usize = TIM2_IRQ_NUM % 32;

    cpu::write!(NVIC.iser[IRQ_INDEX], 1 << IRQ_BIT);
}

#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn handle_tim2_irq() {
    // clear update interrupt flag
    cpu::write!(TIM2.sr[UIF;1], 0);
}

#[cfg(feature = "stm32f072")]
#[inline(never)]
pub fn handle_tim2_irq() {}

#[cfg(feature = "stm32f405")]
#[cfg(target_arch = "arm")]
#[inline(never)]
pub fn current_time() -> MicroSeconds {
    let time_us = cpu::read!(TIM2.cnt);
    MicroSeconds(time_us as u64)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MicroSeconds(pub u64);

impl MicroSeconds {
    pub fn new(microseconds: u64) -> Self {
        MicroSeconds(microseconds)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn sub(self, other: Self) -> MicroSeconds {
        if other.0 > self.0 {
            // wrap-around occurred (every 1 hour)
            let max: u64 = TIME_WRAP_AROUND as u64;
            return MicroSeconds(max - other.0 + self.0 + 1);
        }
        MicroSeconds(self.0 - other.0)
    }
}

#[cfg(feature = "std")]
pub fn current_time() -> MicroSeconds {
    let start = SystemTime::now();
    match start.duration_since(UNIX_EPOCH) {
        Ok(n) => MicroSeconds((n.as_micros() as u64) % ((u32::MAX as u64) * 10_000)),
        Err(_) => panic!("bad system time"),
    }
}
