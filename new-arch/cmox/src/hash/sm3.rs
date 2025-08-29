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
    initialized: bool,
}

impl Sm3 {
    /// Create a new SM3 hasher instance
    pub fn new() -> Self {
        let mut hasher = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            hash_handle: core::ptr::null_mut(),
            initialized: false,
        };

        hasher.init_hash().expect("Failed to initialize SM3 hash");
        hasher
    }

    fn init_hash(&mut self) -> crate::Result<()> {
        ensure_initialized()?;

        // Use the CMOX constructor to set up the handle properly
        self.hash_handle = unsafe { cmox_sm3_construct(&mut self.handle as *mut _) };

        if self.hash_handle.is_null() {
            return Err(crate::error::HashError::Internal.into());
        }

        let result = unsafe { cmox_hash_init(self.hash_handle) };

        HashResult::from_rv(result)?;
        self.initialized = true;
        Ok(())
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

impl FixedOutput for Sm3 {
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

impl Reset for Sm3 {
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

impl Clone for Sm3 {
    fn clone(&self) -> Self {
        // Create a new instance - simple but correct approach
        Self::new()
    }
}

impl Drop for Sm3 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl fmt::Debug for Sm3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sm3")
            .field("initialized", &self.initialized)
            .finish()
    }
}
