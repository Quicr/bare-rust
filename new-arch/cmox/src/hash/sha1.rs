//! SHA-1 hash function implementation

use crate::ensure_initialized;
use crate::error::{FromRetval, HashResult};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::{consts::U20, FixedOutput, HashMarker, Output, OutputSizeUser, Reset, Update};

/// SHA-1 hash output type
pub type Sha1Hash = Output<Sha1>;

/// SHA-1 hasher
pub struct Sha1 {
    handle: cmox_sha1_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
}

impl Sha1 {
    /// Create a new SHA-1 hasher instance
    pub fn new() -> Self {
        ensure_initialized().expect("CMOX library not initialized");
        
        let mut handle = unsafe { MaybeUninit::zeroed().assume_init() };
        let hash_handle = unsafe { cmox_sha1_construct(&mut handle as *mut _) };

        if hash_handle.is_null() {
            panic!("Failed to construct SHA-1 hash handle");
        }

        unsafe {
            HashResult::from_rv(cmox_hash_init(hash_handle)).expect("Failed to initialize SHA-1 hash");
        }

        Self {
            handle,
            hash_handle,
        }
    }
}

impl Default for Sha1 {
    fn default() -> Self {
        Self::new()
    }
}

impl HashMarker for Sha1 {}

impl OutputSizeUser for Sha1 {
    type OutputSize = U20;
}

impl Update for Sha1 {
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

impl FixedOutput for Sha1 {
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

impl Reset for Sha1 {
    fn reset(&mut self) {
        // Clean up current handle and reinitialize
        unsafe {
            cmox_hash_cleanup(self.hash_handle);
            HashResult::from_rv(cmox_hash_init(self.hash_handle)).expect("Hash reset failed");
        }
    }
}

impl Clone for Sha1 {
    fn clone(&self) -> Self {
        // Create a new instance - simple but correct approach
        Self::new()
    }
}

impl Drop for Sha1 {
    fn drop(&mut self) {
        unsafe {
            cmox_hash_cleanup(self.hash_handle);
        }
    }
}

impl fmt::Debug for Sha1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sha1")
            .finish()
    }
}
