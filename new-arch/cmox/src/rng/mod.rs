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
    initialized: bool,
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

        let mut drbg = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            drbg_handle: core::ptr::null_mut(),
            variant,
            initialized: false,
        };

        drbg.init(entropy, nonce, personalization)?;
        Ok(drbg)
    }

    /// Create a CTR-DRBG with reasonable defaults (AES-256, fast implementation)
    ///
    /// # Arguments
    /// * `entropy` - Entropy input (must be at least 32 bytes for AES-256)
    /// * `nonce` - Nonce value (recommended 16 bytes)
    pub fn new_default(entropy: &[u8], nonce: &[u8]) -> crate::Result<Self> {
        Self::new(CtrDrbgVariant::Aes256Fast, entropy, nonce, None)
    }

    fn init(
        &mut self,
        entropy: &[u8],
        nonce: &[u8],
        personalization: Option<&[u8]>,
    ) -> crate::Result<()> {
        ensure_initialized()?;

        // Construct CTR-DRBG handle
        self.drbg_handle =
            unsafe { cmox_ctr_drbg_construct(&mut self.handle, self.variant.to_cmox_impl()) };

        if self.drbg_handle.is_null() {
            return Err(crate::error::DrbgError::Internal.into());
        }

        // Initialize DRBG with entropy, personalization, and nonce
        let (pers_ptr, pers_len) = if let Some(pers) = personalization {
            (pers.as_ptr(), pers.len())
        } else {
            (core::ptr::null(), 0)
        };

        let result = unsafe {
            cmox_drbg_init(
                self.drbg_handle,
                entropy.as_ptr(),
                entropy.len(),
                pers_ptr,
                pers_len,
                nonce.as_ptr(),
                nonce.len(),
            )
        };
        DrbgResult::from_rv(result)?;

        self.initialized = true;
        Ok(())
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
        if !self.initialized {
            return Err(crate::error::DrbgError::UninitializedState.into());
        }

        if output.is_empty() {
            return Ok(());
        }

        let (add_ptr, add_len) = if let Some(additional) = additional_input {
            (additional.as_ptr(), additional.len())
        } else {
            (core::ptr::null(), 0)
        };

        let result = unsafe {
            cmox_drbg_generate(
                self.drbg_handle,
                add_ptr,
                add_len,
                output.as_mut_ptr(),
                output.len(),
            )
        };

        Ok(DrbgResult::from_rv(result)?)
    }

    /// Generate random bytes without additional input
    ///
    /// This is a convenience method that calls `generate` with no additional input.
    pub fn generate_bytes(&mut self, output: &mut [u8]) -> crate::Result<()> {
        self.generate(output, None)
    }

    /// Generate a fixed amount of random bytes and return them
    ///
    /// # Arguments
    /// * `len` - Number of bytes to generate (max 512 bytes per call)
    pub fn generate_vec(&mut self, len: usize) -> crate::Result<heapless::Vec<u8, 512>> {
        if len > 512 {
            return Err(crate::error::DrbgError::Internal.into());
        }

        let mut output = heapless::Vec::new();
        output
            .resize_default(len)
            .map_err(|_| crate::error::DrbgError::Internal)?;
        self.generate_bytes(&mut output)?;
        Ok(output)
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
        if !self.initialized {
            return Err(crate::error::DrbgError::UninitializedState.into());
        }

        // Validate entropy length
        if entropy.len() < self.variant.min_entropy_len() {
            return Err(crate::error::DrbgError::BadEntropySize.into());
        }

        let (add_ptr, add_len) = if let Some(additional) = additional_input {
            (additional.as_ptr(), additional.len())
        } else {
            (core::ptr::null(), 0)
        };

        let result = unsafe {
            cmox_drbg_reseed(
                self.drbg_handle,
                entropy.as_ptr(),
                entropy.len(),
                add_ptr,
                add_len,
            )
        };

        Ok(DrbgResult::from_rv(result)?)
    }

    /// Get the algorithm variant used by this DRBG
    pub fn variant(&self) -> CtrDrbgVariant {
        self.variant
    }

    /// Check if the DRBG is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// A simple helper for creating a CTR-DRBG for testing purposes
///
/// **Warning**: This uses deterministic "entropy" and should NEVER be used in production.
/// It's only suitable for testing and deterministic applications.
impl CtrDrbg {
    /// Create a CTR-DRBG with deterministic entropy for testing
    ///
    /// **SECURITY WARNING**: This function uses predictable entropy and should
    /// NEVER be used in production code. It's only suitable for testing.
    pub fn new_deterministic_for_testing(seed: u64) -> crate::Result<Self> {
        let mut entropy = [0u8; 32];
        let mut nonce = [0u8; 16];

        // Create deterministic but varied entropy from seed
        for i in 0..4 {
            let seed_part = seed.wrapping_add(i as u64);
            entropy[i * 8..(i + 1) * 8].copy_from_slice(&seed_part.to_le_bytes());
        }

        // Create deterministic nonce
        let nonce_seed = seed.wrapping_mul(0x9E3779B97F4A7C15); // Golden ratio hash
        nonce[..8].copy_from_slice(&nonce_seed.to_le_bytes());
        nonce[8..].copy_from_slice(&nonce_seed.wrapping_add(1).to_le_bytes());

        Self::new_default(&entropy, &nonce)
    }
}

impl Drop for CtrDrbg {
    fn drop(&mut self) {
        if self.initialized && !self.drbg_handle.is_null() {
            unsafe {
                cmox_drbg_cleanup(self.drbg_handle);
            }
        }
    }
}

impl RngCore for CtrDrbg {
    fn next_u32(&mut self) -> u32 {
        CtrDrbg::next_u32(self).unwrap_or(0)
    }

    fn next_u64(&mut self) -> u64 {
        CtrDrbg::next_u64(self).unwrap_or(0)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let _ = self.generate_bytes(dest);
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
            .field("initialized", &self.initialized)
            .finish()
    }
}

// CTR-DRBG should not be cloned as each instance must maintain independent state
// for security reasons. Users should create new instances with fresh entropy.
