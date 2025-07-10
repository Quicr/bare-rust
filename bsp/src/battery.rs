//! # Battery Module
//!
//! This module provides functionality for interacting with the battery on the board.
//! It includes methods for initializing the battery interface and retrieving the battery percentage.
//!
//!
//! ## Functions
//!
//! - `get_battery_percentage`: Retrieves the battery percentage.
//!
//!

extern crate hal;

#[cfg(feature = "std")]
extern crate std;

pub struct Battery {}

impl crate::battery::Battery {
    #[inline(never)]
    pub fn new() -> Self {
        crate::battery::Battery {}
    }

    #[inline(never)]
    pub fn init(&self) {}

    /// Retrieves the battery percentage.
    ///
    /// # Returns
    /// A `u8` representing the battery percentage as a number between 0 and 100.
    ///
    pub fn get_battery_percentage(&self) -> u8 {
        99
    }
}
