#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs, rust_2018_idioms)]

//! # CMOX - Idiomatic Rust Cryptography using STM32 CMOX
//!
//! This crate provides idiomatic, type-safe Rust bindings to the STM32 CMOX
//! (Cortex-M Optimized Crypto Stack) library. It implements standard Rust Crypto
//! traits to ensure compatibility with the broader Rust cryptographic ecosystem.

use cmox_sys::{cmox_finalize, cmox_init_arg_t, cmox_initialize, CMOX_INIT_TARGET_AUTO};
use core::sync::atomic::{AtomicBool, Ordering};

pub mod error;

pub mod cipher;
pub mod hash;
pub mod aead;
pub mod mac;
pub mod signature;
pub mod ecdh;
pub mod rng;

pub mod utils;

pub use error::{CmoxError, Result};

// Global initialization tracking
static CMOX_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the CMOX library
///
/// This must be called before using any CMOX cryptographic functions.
/// It's safe to call multiple times - subsequent calls are no-ops.
///
/// # Errors
///
/// Returns an error if the CMOX library fails to initialize.
pub fn initialize() -> Result<()> {
    if CMOX_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }

    let init_arg = cmox_init_arg_t {
        target: CMOX_INIT_TARGET_AUTO,
        pArg: core::ptr::null_mut(),
    };

    let result = unsafe { cmox_initialize(&init_arg as *const _ as *mut _) };

    if result == cmox_sys::CMOX_INIT_SUCCESS {
        CMOX_INITIALIZED.store(true, Ordering::Release);
        Ok(())
    } else {
        Err(CmoxError::InitializationFailed)
    }
}

/// Finalize the CMOX library
///
/// This should be called when shutting down to clean up CMOX resources.
/// After calling this, you must call `initialize()` again before using
/// any CMOX functions.
///
/// # Errors
///
/// Returns an error if the CMOX library fails to finalize properly.
pub fn finalize() -> Result<()> {
    if !CMOX_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }

    let result = unsafe { cmox_finalize(core::ptr::null_mut()) };

    if result == cmox_sys::CMOX_INIT_SUCCESS {
        CMOX_INITIALIZED.store(false, Ordering::Release);
        Ok(())
    } else {
        Err(CmoxError::FinalizationFailed)
    }
}

/// Check if the CMOX library is initialized
pub fn is_initialized() -> bool {
    CMOX_INITIALIZED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        assert!(initialize().is_ok());
        assert!(is_initialized());
        assert!(finalize().is_ok());
        assert!(!is_initialized());
    }
}
