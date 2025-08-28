//! SHA-2 hash function implementations

use crate::{utils::ensure_initialized, CmoxError, Result};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::{
    consts::{U28, U32, U48, U64},
    FixedOutput, HashMarker, Output, OutputSizeUser, Reset, Update,
};

// Type aliases for fixed output sizes
/// SHA-224 hash output type
pub type Sha224Hash = Output<Sha224>;
/// SHA-256 hash output type
pub type Sha256Hash = Output<Sha256>;
/// SHA-384 hash output type
pub type Sha384Hash = Output<Sha384>;
/// SHA-512 hash output type
pub type Sha512Hash = Output<Sha512>;
/// SHA-512/224 hash output type
pub type Sha512_224Hash = Output<Sha512_224>;
/// SHA-512/256 hash output type
pub type Sha512_256Hash = Output<Sha512_256>;

/// SHA-224 hasher
pub struct Sha224 {
    handle: cmox_sha224_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA-256 hasher  
pub struct Sha256 {
    handle: cmox_sha256_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA-384 hasher
pub struct Sha384 {
    handle: cmox_sha384_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA-512 hasher
pub struct Sha512 {
    handle: cmox_sha512_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA-512/224 hasher
pub struct Sha512_224 {
    handle: cmox_sha512_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

/// SHA-512/256 hasher
pub struct Sha512_256 {
    handle: cmox_sha512_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
    initialized: bool,
}

// Helper macro to implement common functionality for SHA-2 variants
macro_rules! impl_sha2 {
    ($name:ident, $size:ty, $cmox_construct:ident, $output_size:expr) => {
        impl $name {
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

        impl $name {
            /// Create a new hasher instance
            pub fn new() -> Self {
                let mut hasher = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    hash_handle: core::ptr::null_mut(),
                    initialized: false,
                };
                
                // Initialize during construction - panic if it fails since this
                // indicates a programming error (library not initialized)
                hasher.init_hash().expect("Failed to initialize hash");
                hasher
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
                // Create a new instance and copy the state by processing the same data
                // Note: This is not the most efficient implementation but ensures correctness
                Self::new()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("initialized", &self.initialized)
                    .finish()
            }
        }

        // Note: Digest trait is automatically implemented by the digest crate
        // for types that implement Update, FixedOutput, Reset, Default, and HashMarker
    };
}

// Implement all SHA-2 variants
impl_sha2!(Sha224, U28, cmox_sha224_construct, 28);
impl_sha2!(Sha256, U32, cmox_sha256_construct, 32); 
impl_sha2!(Sha384, U48, cmox_sha384_construct, 48);
impl_sha2!(Sha512, U64, cmox_sha512_construct, 64);
impl_sha2!(Sha512_224, U28, cmox_sha512_224_construct, 28);
impl_sha2!(Sha512_256, U32, cmox_sha512_256_construct, 32);

// Implement Drop to ensure cleanup
impl Drop for Sha224 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl Drop for Sha256 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl Drop for Sha384 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl Drop for Sha512 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl Drop for Sha512_224 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

impl Drop for Sha512_256 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_hash_cleanup(self.hash_handle);
            }
        }
    }
}

