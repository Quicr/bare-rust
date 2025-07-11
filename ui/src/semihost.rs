//! # Semihost Module
//!
//! This module provides functions for interacting with the semihosting interface,
//! specifically for ARM targets.
//!
//! ## Functions
//!
//! - `exit_no_status`: Exits the application without a status code.
//!
//! ## Usage
//!
//! This module is intended for low-level hardware interaction and should be used with caution.
//! It provides direct access to hardware registers, which can lead to undefined behavior if used incorrectly.
//!


#[cfg(target_arch = "arm")]
use core::arch::asm;

#[cfg(target_arch = "arm")]
#[inline(never)]
pub fn exit_no_status() -> ! {
    unsafe {
        asm!(
            "mov r0, #0x18",
            //"mov r1, #0x20026",
            "movw r1, #0x0026", // Move lower half
            "movt r1, #0x2",    // Move upper half
            "bkpt #0xAB"
        );
    }
    loop {}
}


