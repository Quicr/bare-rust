//! KMAC (Keccak Message Authentication Code) implementation

use crate::{utils::ensure_initialized, CipherError, CmoxError, CoreError, HashError, Result};
use cmox_sys::*;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use digest::{
    generic_array::GenericArray,
    Mac, MacMarker, Output, OutputSizeUser, Reset, VariableOutput, VariableOutputReset,
};

/// KMAC128 implementation (based on SHA-3/Keccak)
pub struct Kmac128 {
    handle: cmox_kmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
    output_size: usize,
}

/// KMAC256 implementation (based on SHA-3/Keccak)
pub struct Kmac256 {
    handle: cmox_kmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
    output_size: usize,
}

// Helper macro to implement KMAC variants
macro_rules! impl_kmac {
    ($name:ident, $default_size:ty, $impl_const:ident, $capacity:expr) => {
        impl MacMarker for $name {}

        impl OutputSizeUser for $name {
            type OutputSize = $default_size;
        }

        impl $name {
            /// Create a new KMAC instance with default output size
            pub fn new(key: &[u8]) -> Result<Self> {
                Self::new_with_size(key, <$default_size>::to_usize())
            }

            /// Create a new KMAC instance with custom output size
            pub fn new_with_size(key: &[u8], output_size: usize) -> Result<Self> {
                let mut kmac = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    mac_handle: core::ptr::null_mut(),
                    initialized: false,
                    output_size,
                };

                kmac.init_with_key(key, output_size)?;
                Ok(kmac)
            }

            /// Create KMAC with customization string
            pub fn new_with_customization(
                key: &[u8], 
                output_size: usize,
                customization: &[u8]
            ) -> Result<Self> {
                let mut kmac = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    mac_handle: core::ptr::null_mut(),
                    initialized: false,
                    output_size,
                };

                kmac.init_with_key_and_customization(key, output_size, customization)?;
                Ok(kmac)
            }

            fn init_with_key(&mut self, key: &[u8], output_size: usize) -> Result<()> {
                ensure_initialized()?;

                // Initialize KMAC handle
                self.mac_handle = unsafe {
                    cmox_kmac_construct(
                        &mut self.handle as *mut _,
                        $impl_const,
                    )
                };

                if self.mac_handle.is_null() {
                    return Err(CmoxError::Cipher(CipherError::InternalError));
                }

                // Initialize MAC
                let result = unsafe {
                    cmox_mac_init(self.mac_handle)
                };
                CmoxError::from_cipher_retval(result)?;

                // Set key
                let result = unsafe {
                    cmox_mac_setKey(
                        self.mac_handle,
                        key.as_ptr(),
                        key.len(),
                    )
                };
                CmoxError::from_cipher_retval(result)?;

                // Set output size
                let result = unsafe {
                    cmox_mac_setTagLen(
                        self.mac_handle,
                        output_size,
                    )
                };
                CmoxError::from_cipher_retval(result)?;

                self.initialized = true;
                self.output_size = output_size;
                Ok(())
            }

            fn init_with_key_and_customization(
                &mut self, 
                key: &[u8], 
                output_size: usize,
                customization: &[u8]
            ) -> Result<()> {
                self.init_with_key(key, output_size)?;

                // Set customization string if KMAC supports it
                // Note: This is a simplified approach - the actual CMOX API might differ
                if !customization.is_empty() {
                    let result = unsafe {
                        cmox_mac_setPersonalizationString(
                            self.mac_handle,
                            customization.as_ptr(),
                            customization.len(),
                        )
                    };
                    CmoxError::from_cipher_retval(result)?;
                }

                Ok(())
            }

            /// Update the MAC with input data
            pub fn update(&mut self, data: &[u8]) -> Result<()> {
                if !self.initialized {
                    return Err(CmoxError::Core(CoreError::NotInitialized));
                }

                if data.is_empty() {
                    return Ok(());
                }

                let result = unsafe {
                    cmox_mac_append(
                        self.mac_handle,
                        data.as_ptr(),
                        data.len(),
                    )
                };

                CmoxError::from_cipher_retval(result)
            }

            /// Finalize and return the MAC tag
            pub fn finalize(self) -> Result<[u8; 128]> {
                if !self.initialized {
                    return Err(CmoxError::Core(CoreError::NotInitialized));
                }

                let mut output = [0u8; 128]; // Max KMAC output size
                let mut output_len = self.output_size.min(128);

                let result = unsafe {
                    cmox_mac_generateTag(
                        self.mac_handle,
                        output.as_mut_ptr(),
                        &mut output_len,
                    )
                };

                CmoxError::from_cipher_retval(result)?;

                // Clean up
                unsafe {
                    cmox_mac_cleanup(self.mac_handle);
                }

                Ok(output)
            }

            /// Finalize into provided buffer
            pub fn finalize_into(self, output: &mut [u8]) -> Result<()> {
                if !self.initialized {
                    return Err(CmoxError::Core(CoreError::NotInitialized));
                }

                let mut output_len = output.len();

                let result = unsafe {
                    cmox_mac_generateTag(
                        self.mac_handle,
                        output.as_mut_ptr(),
                        &mut output_len,
                    )
                };

                CmoxError::from_cipher_retval(result)?;

                // Clean up
                unsafe {
                    cmox_mac_cleanup(self.mac_handle);
                }

                Ok(())
            }
        }

        impl Mac for $name {
            fn new(key: &digest::Key<Self>) -> Self {
                Self::new(key.as_slice()).expect("Failed to initialize KMAC")
            }

            fn new_from_slice(key: &[u8]) -> digest::Result<Self> {
                Self::new(key).map_err(|_| digest::InvalidLength)
            }

            fn update(&mut self, data: &[u8]) {
                self.update(data).expect("KMAC update failed");
            }

            fn finalize(self) -> Output<Self> {
                let result = self.finalize().expect("KMAC finalization failed");
                let mut output = Output::<Self>::default();
                let copy_len = core::cmp::min(result.len(), output.len());
                output[..copy_len].copy_from_slice(&result[..copy_len]);
                output
            }

            fn reset(&mut self) {
                if self.initialized {
                    unsafe {
                        cmox_mac_cleanup(self.mac_handle);
                    }
                    self.initialized = false;
                }

                panic!("KMAC reset requires re-initialization with key");
            }
        }

        impl VariableOutput for $name {
            const MAX_OUTPUT_SIZE: usize = $capacity / 8;

            fn new(output_size: usize) -> Result<Self, digest::InvalidOutputSize> {
                if output_size > Self::MAX_OUTPUT_SIZE {
                    return Err(digest::InvalidOutputSize);
                }
                
                // For variable output, we need a key - this is a limitation of the MAC trait
                // In practice, use new_with_size instead
                Err(digest::InvalidOutputSize)
            }

            fn output_size(&self) -> usize {
                self.output_size
            }

            fn finalize_variable(self, output: &mut [u8]) -> Result<(), digest::InvalidBufferSize> {
                if output.len() != self.output_size {
                    return Err(digest::InvalidBufferSize);
                }
                
                self.finalize_into(output).map_err(|_| digest::InvalidBufferSize)
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                if self.initialized {
                    unsafe {
                        cmox_mac_cleanup(self.mac_handle);
                    }
                }
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                panic!("Clone not supported for initialized KMAC - create new instance");
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("initialized", &self.initialized)
                    .field("output_size", &self.output_size)
                    .finish()
            }
        }
    };
}

// Implement KMAC variants
impl_kmac!(Kmac128, digest::consts::U32, CMOX_KMAC128, 1600); // KMAC128 default 256-bit output, capacity 1600
impl_kmac!(Kmac256, digest::consts::U64, CMOX_KMAC256, 1600); // KMAC256 default 512-bit output, capacity 1600