//! SHA-1 hash function implementation

use crate::ensure_initialized;
use crate::error::{FromRetval, HashError, HashResult, Result};
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
    initialized: bool,
}

impl Sha1 {
    /// Create a new SHA-1 hasher instance
    pub fn new() -> Self {
        let mut hasher = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            hash_handle: core::ptr::null_mut(),
            initialized: false,
        };

        hasher.init_hash().expect("Failed to initialize SHA-1 hash");
        hasher
    }

    fn init_hash(&mut self) -> Result<()> {
        ensure_initialized()?;

        // Use the CMOX constructor to set up the handle properly
        self.hash_handle = unsafe { cmox_sha1_construct(&mut self.handle as *mut _) };

        if self.hash_handle.is_null() {
            return Err(HashError::Internal.into());
        }

        let result = unsafe { cmox_hash_init(self.hash_handle) };

        HashResult::from_rv(result)?;
        self.initialized = true;
        Ok(())
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
        if !self.initialized {
            panic!("Hash not initialized");
        }

        if data.is_empty() {
            return;
        }

        let result = unsafe { cmox_hash_append(self.hash_handle, data.as_ptr(), data.len()) };

        HashResult::from_rv(result).expect("Hash update failed");
    }
}

impl FixedOutput for Sha1 {
    fn finalize_into(self, out: &mut Output<Self>) {
        if !self.initialized {
            panic!("Hash not initialized");
        }

        let mut digest_len = out.len();
        let result = unsafe {
            cmox_hash_generateTag(
                self.hash_handle,
                out.as_mut_ptr(),
                &mut digest_len as *mut usize,
            )
        };

        HashResult::from_rv(result).expect("Hash finalization failed");

        // Clean up the handle
        unsafe {
            cmox_hash_cleanup(self.hash_handle);
        }
    }
}

impl Reset for Sha1 {
    fn reset(&mut self) {
        // Clean up current handle
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }

        self.init_hash().expect("Hash reset failed");
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
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl fmt::Debug for Sha1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sha1")
            .field("initialized", &self.initialized)
            .finish()
    }
}
