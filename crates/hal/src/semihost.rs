//! # Semihost Module
//!
//! This module provides functions for interacting with the semihosting interface, 
//! specifically for ARM targets.
//! It includes functions for exiting the application with or without a status code.
//!
//! ## Functions
//!
//! - `exit_no_status`: Exits the application without a status code.
//! - `exit`: Exits the application with a specified status code.
//!
//! ## Usage
//!
//! This module is intended for low-level hardware interaction and should be used with caution.
//! It provides direct access to hardware registers, which can lead to undefined behavior if used incorrectly.
//!

#[cfg(feature = "std")]
extern crate std;

#[cfg(target_arch = "arm")]
use core::arch::asm;

#[cfg(target_arch = "arm")]
#[inline]
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

#[cfg(target_arch = "arm")]
pub fn exit(ret: i32) -> ! {
    #[repr(C, packed)]
    struct Args {
        reason: u32,
        status: i32,
    }
    const ADP_STOPPED_APPLICATION_EXIT: u32 = 0x20026; // from https://github.com/ARM-software/abi-aa/blob/main/semihosting/semihosting.rst#sys-exit-0x18
    let args = Args {
        reason: ADP_STOPPED_APPLICATION_EXIT,
        status: ret,
    };
    let arg_ptr: *const Args = &args;

    unsafe {
        asm!(
        "mov r0, #0x20", // from https://github.com/ARM-software/abi-aa/blob/main/semihosting/semihosting.rst#66sys_exit_extended-0x20
        "mov r1, {0}",
        "bkpt #0xAB", // from https://github.com/ARM-software/abi-aa/blob/main/semihosting/semihosting.rst#4the-semihosting-interface
        in(reg) arg_ptr,
        );
    }
    loop {}
}

#[cfg(feature = "std")]
pub fn exit(ret: i32) -> ! {
    std::process::exit(ret);
}
