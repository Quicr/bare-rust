//! SM3 hash function implementation

use crate::ensure_initialized;
use crate::error::{FromRetval, HashResult};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::{consts::U32, FixedOutput, HashMarker, Output, OutputSizeUser, Reset, Update};

/// SM3 hash output type
pub type Sm3Hash = Output<Sm3>;

/// SM3 hasher
pub struct Sm3 {
    handle: cmox_sm3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
}

impl Sm3 {
    /// Create a new SM3 hasher instance
    pub fn new() -> Self {
        ensure_initialized().expect("CMOX library not initialized");
        
        let mut handle = unsafe { MaybeUninit::zeroed().assume_init() };
        let hash_handle = unsafe { cmox_sm3_construct(&mut handle as *mut _) };

        if hash_handle.is_null() {
            panic!("Failed to construct SM3 hash handle");
        }

        unsafe {
            HashResult::from_rv(cmox_hash_init(hash_handle))
                .expect("Failed to initialize SM3 hash");
        }

        Self {
            handle,
            hash_handle,
        }
    }
}

impl Default for Sm3 {
    fn default() -> Self {
        Self::new()
    }
}

impl HashMarker for Sm3 {}

impl OutputSizeUser for Sm3 {
    type OutputSize = U32;
}

impl Update for Sm3 {
    fn update(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        unsafe {
            HashResult::from_rv(cmox_hash_append(self.hash_handle, data.as_ptr(), data.len()))
                .expect("Hash update failed");
        }
    }
}

impl FixedOutput for Sm3 {
    fn finalize_into(self, out: &mut Output<Self>) {
        let mut digest_len = out.len();
        unsafe {
            HashResult::from_rv(cmox_hash_generateTag(
                self.hash_handle,
                out.as_mut_ptr(),
                &mut digest_len as *mut usize,
            ))
            .expect("Hash finalization failed");

            // Clean up the handle
            cmox_hash_cleanup(self.hash_handle);
        }
    }
}

impl Reset for Sm3 {
    fn reset(&mut self) {
        // Clean up current handle and reinitialize
        unsafe {
            cmox_hash_cleanup(self.hash_handle);
            HashResult::from_rv(cmox_hash_init(self.hash_handle))
                .expect("Hash reset failed");
        }
    }
}

impl Clone for Sm3 {
    fn clone(&self) -> Self {
        // Create a new instance - simple but correct approach
        Self::new()
    }
}

impl Drop for Sm3 {
    fn drop(&mut self) {
        unsafe {
            cmox_hash_cleanup(self.hash_handle);
        }
    }
}

impl fmt::Debug for Sm3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sm3")
            .finish()
    }
}
