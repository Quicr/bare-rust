#![no_std]
#![no_main]
//#![doc = include_str!("../README.md")]
#![warn(missing_docs, rust_2018_idioms)]
#![allow(clippy::len_without_is_empty)]
#![allow(dead_code)] // XXX(RLB) Only while reviewing / refactoring

//! # CMOX - Idiomatic Rust Cryptography using STM32 CMOX
//!
//! This crate provides idiomatic, type-safe Rust bindings to the STM32 CMOX
//! (Cortex-M Optimized Crypto Stack) library. It implements standard Rust Crypto
//! traits to ensure compatibility with the broader Rust cryptographic ecosystem.
//!
//! ## STM32 Family Selection
//!
//! By default, the crate uses `stm32-auto` which auto-detects the STM32 target.
//! For better performance and smaller code size, enable a specific STM32 family feature:
//!
//! ```toml
//! [dependencies]
//! cmox = { version = "0.1", default-features = false, features = ["stm32h7"] }
//! ```
//!
//! Available STM32 family features: `stm32f0`, `stm32f1`, `stm32f2`, `stm32f3`,
//! `stm32f4`, `stm32f7`, `stm32g0`, `stm32g4`, `stm32h5`, `stm32h7`, `stm32h7ab`,
//! `stm32l0`, `stm32l1`, `stm32l4`, `stm32l5`, `stm32wb`, `stm32wba`, `stm32wl`.
//!
//! Each feature also enables the appropriate Cortex-M core features in the underlying
//! `cmox-sys` crate for optimal code generation.

use cmox_sys::*;

// Conditional imports based on enabled features to avoid unused import warnings
#[cfg(feature = "stm32-auto")]
use cmox_sys::CMOX_INIT_TARGET_AUTO as CMOX_INIT_TARGET;
#[cfg(feature = "stm32f0")]
use cmox_sys::CMOX_INIT_TARGET_F0 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32f1")]
use cmox_sys::CMOX_INIT_TARGET_F1 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32f2")]
use cmox_sys::CMOX_INIT_TARGET_F2 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32f3")]
use cmox_sys::CMOX_INIT_TARGET_F3 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32f4")]
use cmox_sys::CMOX_INIT_TARGET_F4 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32f7")]
use cmox_sys::CMOX_INIT_TARGET_F7 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32g0")]
use cmox_sys::CMOX_INIT_TARGET_G0 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32g4")]
use cmox_sys::CMOX_INIT_TARGET_G4 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32h5")]
use cmox_sys::CMOX_INIT_TARGET_H5 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32h7")]
use cmox_sys::CMOX_INIT_TARGET_H7 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32h7ab")]
use cmox_sys::CMOX_INIT_TARGET_H7AB as CMOX_INIT_TARGET;
#[cfg(feature = "stm32l0")]
use cmox_sys::CMOX_INIT_TARGET_L0 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32l1")]
use cmox_sys::CMOX_INIT_TARGET_L1 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32l4")]
use cmox_sys::CMOX_INIT_TARGET_L4 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32l5")]
use cmox_sys::CMOX_INIT_TARGET_L5 as CMOX_INIT_TARGET;
#[cfg(feature = "stm32wb")]
use cmox_sys::CMOX_INIT_TARGET_WB as CMOX_INIT_TARGET;
#[cfg(feature = "stm32wba")]
use cmox_sys::CMOX_INIT_TARGET_WBA as CMOX_INIT_TARGET;
#[cfg(feature = "stm32wl")]
use cmox_sys::CMOX_INIT_TARGET_WL as CMOX_INIT_TARGET;

use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

pub mod error;
pub mod utils;

pub mod aead;
pub mod drbg;
pub mod ecdh;
pub mod hash;
pub mod mac;
//pub mod signature;

pub use error::{
    CipherError, CmoxError, CoreError, DrbgError, EccError, HashError, Result, RsaError,
};

use error::{CoreResult, FromRetval};

// Global initialization tracking
static CMOX_INITIALIZED: AtomicBool = AtomicBool::new(false);

// Dummy functions to make the init/finalize functions happy
#[no_mangle]
extern "C" fn cmox_ll_init(_p_arg: *const c_void) -> cmox_init_retval_t {
    CMOX_INIT_SUCCESS
}

#[no_mangle]
extern "C" fn cmox_ll_deInit(_p_arg: *const c_void) -> cmox_init_retval_t {
    CMOX_INIT_SUCCESS
}

/// Query the CMOX library for its version
pub fn version() -> u32 {
    let mut info = unsafe { MaybeUninit::zeroed().assume_init() };
    unsafe { cmox_getInfos(&mut info) };
    info.version
}

/// Initialize the CMOX library
///
/// This must be called before using any CMOX cryptographic functions.
/// It's safe to call multiple times - subsequent calls are no-ops.
///
/// The initialization target is determined by the enabled Cargo features:
/// - `stm32-auto` (default): Auto-detect the target MCU
/// - `stm32f0`, `stm32f1`, etc.: Use specific STM32 family optimization
///
/// Using a specific STM32 family feature can provide better performance and smaller
/// code size compared to auto-detection, and also enables appropriate Cortex-M
/// core optimizations in the underlying `cmox-sys` crate.
///
/// # Errors
///
/// Returns an error if the CMOX library fails to initialize.
pub fn initialize() -> Result<()> {
    if CMOX_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }

    let init_arg = cmox_init_arg_t {
        target: CMOX_INIT_TARGET,
        pArg: core::ptr::null_mut(),
    };

    unsafe { CoreResult::from_rv(cmox_initialize(&init_arg as *const _ as *mut _))? };

    CMOX_INITIALIZED.store(true, Ordering::Release);
    Ok(())
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

    unsafe { CoreResult::from_rv(cmox_finalize(core::ptr::null_mut()))? };
    CMOX_INITIALIZED.store(false, Ordering::Release);
    Ok(())
}

/// Check if the CMOX library is initialized
pub fn is_initialized() -> bool {
    CMOX_INITIALIZED.load(Ordering::Acquire)
}

/// Ensure CMOX library is initialized before calling cryptographic functions
pub(crate) fn ensure_initialized() -> Result<()> {
    if !is_initialized() {
        initialize()
    } else {
        Ok(())
    }
}
