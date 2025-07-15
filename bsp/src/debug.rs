//! # Debug Module
//!
//! This module provides functionality for initializing and controlling debug pins for the board.
//!
//!
//! ## Functions
//!
//! - `set`: Sets the state of a specified debug pin.
//!
//! ## Example
//!
//! ```rust
//!  use bsp::BSP;
//!  use bsp::debug;
//!  let mut bsp = BSP::new();
//!  bsp.init();
//!
//! // Set the debug pin 0 to high
//! bsp::debug::set(0, true);
//! ```
//!

extern crate hal;

use crate::board;

pub struct Debug {}

impl Debug {
    #[inline(never)]
    pub fn new() -> Self {
        Debug {}
    }

    #[inline(never)]
    pub fn init(&self) {
        board::info::DEBUG1_PIN.output();
        board::info::DEBUG1_PIN.low();
    }
}

/// Sets the state of a specified debug pin.
///
/// This function sets the state of the debug pin identified by the given channel number.
/// The pin can be set to either high or low based on the boolean value provided.
///
/// # Arguments
///
/// * `ch` - The channel number of the debug pin. Currently only 0 is supported.
/// * `v` - The state to set the debug pin to (`true` for high, `false` for low).
///
/// # Panics
///
/// This function will panic if the channel number is not 0.
///
/// # Example
///
/// ```rust
/// use bsp::BSP;
/// use bsp::debug;
/// let mut bsp = BSP::new();
/// bsp.init();
///
/// // Set the debug pin 0 to high
/// debug::set(0, true);
///
/// // Set the debug pin 0 to low
/// debug::set(0, false);
/// ```
///
pub fn set(channel: u8, state: bool) {
    assert!(channel < 1); // TODO - implement up to 4

    if board::info::NUM_DEBUG_PINS >= 1 {
        if state {
            board::info::DEBUG1_PIN.high();
        } else {
            board::info::DEBUG1_PIN.low();
        }
    }
}
