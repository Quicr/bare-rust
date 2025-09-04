//! Cryptographically Secure Random Number Generation using CMOX library
//!
//! This module provides CTR-DRBG (Counter mode Deterministic Random Bit Generator)
//! implementation using the STM32 CMOX library. CTR-DRBG is specified in NIST SP 800-90A
//! and provides cryptographically secure pseudorandom number generation.
//!
//! ## Features
//!
//! - **CTR-DRBG**: NIST SP 800-90A compliant deterministic random bit generator
//! - **AES-based**: Uses AES-128 or AES-256 in counter mode for security
//! - **Proper Seeding**: Supports entropy input, nonce, and personalization string
//! - **Reseeding**: Periodic reseeding for long-running applications
//! - **Security Features**:
//!   - Cryptographically secure output
//!   - Proper state management
//!   - Prediction resistance when reseeded
//!   - Memory-safe operations

use crate::ensure_initialized;
use crate::error::{DrbgResult, FromRetval};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use core::num::NonZeroU32;
use rand_core::{CryptoRng, RngCore};

/// CTR-DRBG algorithm variants
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CtrDrbgVariant {
    /// AES-128 based CTR-DRBG (fast implementation)
    Aes128Fast,
    /// AES-128 based CTR-DRBG (small implementation)
    Aes128Small,
    /// AES-256 based CTR-DRBG (fast implementation)
    Aes256Fast,
    /// AES-256 based CTR-DRBG (small implementation)
    Aes256Small,
}

impl CtrDrbgVariant {
    fn to_cmox_impl(self) -> cmox_ctr_drbg_impl_t {
        match self {
            CtrDrbgVariant::Aes128Fast => unsafe { CMOX_CTR_DRBG_AES128_FAST },
            CtrDrbgVariant::Aes128Small => unsafe { CMOX_CTR_DRBG_AES128_SMALL },
            CtrDrbgVariant::Aes256Fast => unsafe { CMOX_CTR_DRBG_AES256_FAST },
            CtrDrbgVariant::Aes256Small => unsafe { CMOX_CTR_DRBG_AES256_SMALL },
        }
    }

    /// Get minimum entropy length required for this variant
    pub fn min_entropy_len(self) -> usize {
        match self {
            CtrDrbgVariant::Aes128Fast | CtrDrbgVariant::Aes128Small => 16, // AES-128 requires 16 bytes minimum
            CtrDrbgVariant::Aes256Fast | CtrDrbgVariant::Aes256Small => 32, // AES-256 requires 32 bytes minimum
        }
    }

    /// Get recommended nonce length for this variant
    pub fn recommended_nonce_len(self) -> usize {
        match self {
            CtrDrbgVariant::Aes128Fast | CtrDrbgVariant::Aes128Small => 8, // 8 bytes nonce
            CtrDrbgVariant::Aes256Fast | CtrDrbgVariant::Aes256Small => 16, // 16 bytes nonce
        }
    }
}

/// CTR-DRBG (Counter mode Deterministic Random Bit Generator)
///
/// A cryptographically secure random number generator based on AES in counter mode
/// as specified in NIST SP 800-90A.
pub struct CtrDrbg {
    handle: cmox_ctr_drbg_handle_t,
    drbg_handle: *mut cmox_drbg_handle_t,
    variant: CtrDrbgVariant,
}

impl CtrDrbg {
    /// Create a new CTR-DRBG instance with specified algorithm variant
    ///
    /// # Arguments
    /// * `variant` - The CTR-DRBG algorithm variant to use
    /// * `entropy` - Entropy input (seed material) - must meet minimum length requirements
    /// * `nonce` - Nonce value for additional randomness
    /// * `personalization` - Optional personalization string for domain separation
    pub fn new(
        variant: CtrDrbgVariant,
        entropy: &[u8],
        nonce: &[u8],
        personalization: Option<&[u8]>,
    ) -> crate::Result<Self> {
        // Validate entropy length
        if entropy.len() < variant.min_entropy_len() {
            return Err(crate::error::DrbgError::BadEntropySize.into());
        }

        ensure_initialized()?;

        let mut handle = unsafe { MaybeUninit::zeroed().assume_init() };
        let drbg_handle = unsafe { cmox_ctr_drbg_construct(&mut handle, variant.to_cmox_impl()) };

        if drbg_handle.is_null() {
            return Err(crate::error::DrbgError::Internal.into());
        }

        // Initialize DRBG with entropy, personalization, and nonce
        let (pers_ptr, pers_len) = if let Some(pers) = personalization {
            (pers.as_ptr(), pers.len())
        } else {
            (core::ptr::null(), 0)
        };

        unsafe {
            DrbgResult::from_rv(cmox_drbg_init(
                drbg_handle,
                entropy.as_ptr(),
                entropy.len(),
                pers_ptr,
                pers_len,
                nonce.as_ptr(),
                nonce.len(),
            ))?;
        }

        Ok(Self {
            handle,
            drbg_handle,
            variant,
        })
    }

    /// Create a CTR-DRBG with reasonable defaults (AES-256, fast implementation)
    ///
    /// # Arguments
    /// * `entropy` - Entropy input (must be at least 32 bytes for AES-256)
    /// * `nonce` - Nonce value (recommended 16 bytes)
    pub fn new_default(entropy: &[u8], nonce: &[u8]) -> crate::Result<Self> {
        Self::new(CtrDrbgVariant::Aes256Fast, entropy, nonce, None)
    }

    /// Generate random bytes
    ///
    /// # Arguments
    /// * `output` - Buffer to fill with random bytes
    /// * `additional_input` - Optional additional input for prediction resistance
    pub fn generate(
        &mut self,
        output: &mut [u8],
        additional_input: Option<&[u8]>,
    ) -> crate::Result<()> {
        if output.is_empty() {
            return Ok(());
        }

        let (add_ptr, add_len) = if let Some(additional) = additional_input {
            (additional.as_ptr(), additional.len())
        } else {
            (core::ptr::null(), 0)
        };

        unsafe {
            Ok(DrbgResult::from_rv(cmox_drbg_generate(
                self.drbg_handle,
                add_ptr,
                add_len,
                output.as_mut_ptr(),
                output.len(),
            ))?)
        }
    }

    /// Generate random bytes without additional input
    ///
    /// This is a convenience method that calls `generate` with no additional input.
    pub fn generate_bytes(&mut self, output: &mut [u8]) -> crate::Result<()> {
        self.generate(output, None)
    }

    /// Generate a random u32
    pub fn next_u32(&mut self) -> crate::Result<u32> {
        let mut bytes = [0u8; 4];
        self.generate_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Generate a random u64
    pub fn next_u64(&mut self) -> crate::Result<u64> {
        let mut bytes = [0u8; 8];
        self.generate_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    /// Reseed the DRBG with additional entropy
    ///
    /// This should be called periodically for long-running applications to maintain
    /// security properties and handle potential state compromise.
    ///
    /// # Arguments
    /// * `entropy` - Fresh entropy input (must meet minimum length requirements)
    /// * `additional_input` - Optional additional input for domain separation
    pub fn reseed(&mut self, entropy: &[u8], additional_input: Option<&[u8]>) -> crate::Result<()> {
        // Validate entropy length
        if entropy.len() < self.variant.min_entropy_len() {
            return Err(crate::error::DrbgError::BadEntropySize.into());
        }

        let (add_ptr, add_len) = if let Some(additional) = additional_input {
            (additional.as_ptr(), additional.len())
        } else {
            (core::ptr::null(), 0)
        };

        unsafe {
            Ok(DrbgResult::from_rv(cmox_drbg_reseed(
                self.drbg_handle,
                entropy.as_ptr(),
                entropy.len(),
                add_ptr,
                add_len,
            ))?)
        }
    }

    /// Get the algorithm variant used by this DRBG
    pub fn variant(&self) -> CtrDrbgVariant {
        self.variant
    }
}

impl Drop for CtrDrbg {
    fn drop(&mut self) {
        if !self.drbg_handle.is_null() {
            unsafe {
                cmox_drbg_cleanup(self.drbg_handle);
            }
        }
    }
}

impl RngCore for CtrDrbg {
    fn next_u32(&mut self) -> u32 {
        self.next_u32().expect("DRBG failure in next_u32")
    }

    fn next_u64(&mut self) -> u64 {
        self.next_u64().expect("DRBG failure in next_u64")
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.generate_bytes(dest).expect("DRBG failure in next_u64")
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> core::result::Result<(), rand_core::Error> {
        match self.generate_bytes(dest) {
            Ok(()) => Ok(()),
            Err(_) => Err(rand_core::Error::from(NonZeroU32::new(1).unwrap())),
        }
    }
}

impl CryptoRng for CtrDrbg {}

impl fmt::Debug for CtrDrbg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CtrDrbg")
            .field("variant", &self.variant)
            .finish()
    }
}
