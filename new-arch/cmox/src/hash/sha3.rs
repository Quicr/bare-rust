//! SHA-3 hash function implementations

use crate::ensure_initialized;
use crate::error::{FromRetval, HashResult};
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
}

/// SHA3-256 hasher
pub struct Sha3_256 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
}

/// SHA3-384 hasher
pub struct Sha3_384 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
}

/// SHA3-512 hasher
pub struct Sha3_512 {
    handle: cmox_sha3_handle_t,
    hash_handle: *mut cmox_hash_handle_t,
}

// Helper macro to implement common functionality for SHA-3 variants
macro_rules! impl_sha3 {
    ($name:ident, $size:ty, $cmox_construct:ident, $output_size:expr) => {
        impl $name {
            /// Create a new hasher instance
            pub fn new() -> Self {
                ensure_initialized().expect("CMOX library not initialized");
                
                let mut handle = unsafe { MaybeUninit::zeroed().assume_init() };
                let hash_handle = unsafe { $cmox_construct(&mut handle as *mut _) };

                if hash_handle.is_null() {
                    panic!("Failed to construct hash handle");
                }

                unsafe {
                    HashResult::from_rv(cmox_hash_init(hash_handle))
                        .expect("Failed to initialize hash");
                }

                Self {
                    handle,
                    hash_handle,
                }
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
                if data.is_empty() {
                    return;
                }

                unsafe {
                    HashResult::from_rv(cmox_hash_append(self.hash_handle, data.as_ptr(), data.len()))
                        .expect("Hash update failed");
                }
            }
        }

        impl FixedOutput for $name {
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

        impl Reset for $name {
            fn reset(&mut self) {
                // Clean up current handle and reinitialize
                unsafe {
                    cmox_hash_cleanup(self.hash_handle);
                    HashResult::from_rv(cmox_hash_init(self.hash_handle))
                        .expect("Hash reset failed");
                }
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
                unsafe {
                    cmox_hash_cleanup(self.hash_handle);
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
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
