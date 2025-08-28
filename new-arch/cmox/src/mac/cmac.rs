//! CMAC (Cipher-based Message Authentication Code) implementation

use crate::{utils::ensure_initialized, CmoxError, Result};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::{
    generic_array::GenericArray,
    Mac, MacMarker, Output, OutputSizeUser, Reset,
};

/// AES-CMAC implementation
pub struct AesCmac {
    handle: cmox_cmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
}

/// SM4-CMAC implementation  
pub struct Sm4Cmac {
    handle: cmox_cmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
}

// Helper macro to implement CMAC variants
macro_rules! impl_cmac {
    ($name:ident, $output_size:ty, $impl_const:ident) => {
        impl MacMarker for $name {}

        impl OutputSizeUser for $name {
            type OutputSize = $output_size;
        }

        impl $name {
            fn init_with_key(&mut self, key: &[u8]) -> Result<()> {
                ensure_initialized()?;

                // Initialize CMAC handle
                self.mac_handle = unsafe {
                    cmox_cmac_construct(
                        &mut self.handle as *mut _,
                        $impl_const,
                    )
                };

                if self.mac_handle.is_null() {
                    return Err(CmoxError::InitializationFailed);
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

                self.initialized = true;
                Ok(())
            }
        }

        impl Mac for $name {
            fn new(key: &digest::Key<Self>) -> Self {
                let mut cmac = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    mac_handle: core::ptr::null_mut(),
                    initialized: false,
                };

                cmac.init_with_key(key.as_slice()).expect("Failed to initialize CMAC");
                cmac
            }

            fn new_from_slice(key: &[u8]) -> digest::Result<Self> {
                let mut cmac = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    mac_handle: core::ptr::null_mut(),
                    initialized: false,
                };

                cmac.init_with_key(key).map_err(|_| digest::InvalidLength)?;
                Ok(cmac)
            }

            fn update(&mut self, data: &[u8]) {
                if !self.initialized {
                    panic!("CMAC not initialized");
                }

                if data.is_empty() {
                    return;
                }

                let result = unsafe {
                    cmox_mac_append(
                        self.mac_handle,
                        data.as_ptr(),
                        data.len(),
                    )
                };

                CmoxError::from_cipher_retval(result).expect("CMAC update failed");
            }

            fn finalize(self) -> Output<Self> {
                if !self.initialized {
                    panic!("CMAC not initialized");
                }

                let mut output = Output::<Self>::default();
                let mut output_len = output.len();

                let result = unsafe {
                    cmox_mac_generateTag(
                        self.mac_handle,
                        output.as_mut_ptr(),
                        &mut output_len,
                    )
                };

                CmoxError::from_cipher_retval(result).expect("CMAC finalization failed");

                // Clean up
                unsafe {
                    cmox_mac_cleanup(self.mac_handle);
                }

                output
            }

            fn reset(&mut self) {
                if self.initialized {
                    unsafe {
                        cmox_mac_cleanup(self.mac_handle);
                    }
                    self.initialized = false;
                }

                panic!("CMAC reset requires re-initialization with key");
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
                panic!("Clone not supported for initialized CMAC - create new instance");
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

// Implement CMAC variants
impl_cmac!(AesCmac, digest::consts::U16, CMOX_AESCMAC);  // AES-CMAC produces 128-bit tags
impl_cmac!(Sm4Cmac, digest::consts::U16, CMOX_SM4CMAC);  // SM4-CMAC produces 128-bit tags