//! SHA-3 hash function implementations

use crate::{utils::ensure_initialized, CmoxError, Result};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::{
    consts::{U28, U32, U48, U64},
    FixedOutput, HashMarker, Output, OutputSizeUser, Reset, Update,
};

// Type aliases for fixed output sizes
/// SHA3-224 hash output type
pub type Sha3_224Hash = Output<Sha3_224>;
/// SHA3-256 hash output type  
pub type Sha3_256Hash = Output<Sha3_256>;
/// SHA3-384 hash output type
pub type Sha3_384Hash = Output<Sha3_384>;
/// SHA3-512 hash output type
pub type Sha3_512Hash = Output<Sha3_512>;

/// SHA3-224 hasher
pub struct Sha3_224 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA3-256 hasher
pub struct Sha3_256 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA3-384 hasher
pub struct Sha3_384 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA3-512 hasher
pub struct Sha3_512 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

// Helper macro to implement common functionality for SHA-3 variants
macro_rules! impl_sha3 {
    ($name:ident, $size:ty, $cmox_construct:ident, $output_size:expr) => {
        impl $name {
            /// Create a new hasher instance
            pub fn new() -> Self {
                let mut hasher = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    hash_handle: core::ptr::null_mut(),
                    initialized: false,
                };
                
                hasher.init_hash().expect("Failed to initialize hash");
                hasher
            }

            fn init_hash(&mut self) -> Result<()> {
                ensure_initialized()?;
                
                // Use the CMOX constructor to set up the handle properly
                self.hash_handle = unsafe { 
                    $cmox_construct(&mut self.handle as *mut _)
                };
                
                if self.hash_handle.is_null() {
                    return Err(CmoxError::InitializationFailed);
                }

                let result = unsafe {
                    cmox_hash_init(self.hash_handle)
                };
                
                CmoxError::from_hash_retval(result)?;
                self.initialized = true;
                Ok(())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl HashMarker for $name {}

        impl OutputSizeUser for $name {
            type OutputSize = $size;
        }

        impl Update for $name {
            fn update(&mut self, data: &[u8]) {
                if !self.initialized {
                    panic!("Hash not initialized");
                }
                
                if data.is_empty() {
                    return;
                }
                
                let result = unsafe {
                    cmox_hash_append(
                        self.hash_handle,
                        data.as_ptr(),
                        data.len(),
                    )
                };
                
                CmoxError::from_hash_retval(result).expect("Hash update failed");
            }
        }

        impl FixedOutput for $name {
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
                
                CmoxError::from_hash_retval(result).expect("Hash finalization failed");
                
                // Clean up the handle
                unsafe {
                    cmox_hash_cleanup(self.hash_handle);
                }
            }
        }

        impl Reset for $name {
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

        impl Clone for $name {
            fn clone(&self) -> Self {
                // Create a new instance - simple but correct approach
                Self::new()
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if self.initialized {
                    unsafe {
                        cmox_hash_cleanup(self.hash_handle);
                    }
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("initialized", &self.initialized)
                    .finish()
            }
        }
    };
}

// Implement all SHA-3 variants
impl_sha3!(Sha3_224, U28, cmox_sha3_224_construct, 28);
impl_sha3!(Sha3_256, U32, cmox_sha3_256_construct, 32);
impl_sha3!(Sha3_384, U48, cmox_sha3_384_construct, 48);
impl_sha3!(Sha3_512, U64, cmox_sha3_512_construct, 64);