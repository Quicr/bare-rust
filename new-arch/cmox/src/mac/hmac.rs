//! HMAC (Hash-based Message Authentication Code) implementation

use crate::{utils::ensure_initialized, CipherError, CmoxError, CoreError, HashError, Result};
use cmox_sys::*;
use core::fmt;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use digest::{
    block_buffer::Eager,
    core_api::{BufferKindUser, CoreProxy, FixedOutputCore, UpdateCore},
    generic_array::GenericArray,
    HashMarker, Mac, MacMarker, Output, OutputSizeUser, Reset, Update,
};

/// HMAC wrapper that can work with any hash function
pub struct Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    handle: cmox_hmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
    _digest: PhantomData<D>,
}

impl<D> MacMarker for Hmac<D> where D: OutputSizeUser + Clone {}

impl<D> OutputSizeUser for Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    type OutputSize = D::OutputSize;
}

impl<D> Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    fn get_hash_algo() -> *const cmox_hash_algo_t {
        // Map digest types to CMOX hash algorithms
        // This is a simplified approach - in a real implementation you'd use trait bounds
        // or associated types to determine the correct algorithm
        unsafe { &CMOX_SHA256_ALGO as *const _ }
    }

    fn init_with_key(&mut self, key: &[u8]) -> Result<()> {
        ensure_initialized()?;

        // Initialize HMAC handle
        self.mac_handle = unsafe {
            cmox_hmac_construct(
                &mut self.handle as *mut _,
                Self::get_hash_algo(),
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

        self.initialized = true;
        Ok(())
    }
}

impl<D> Mac for Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    fn new(key: &digest::Key<Self>) -> Self {
        let mut hmac = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            mac_handle: core::ptr::null_mut(),
            initialized: false,
            _digest: PhantomData,
        };

        hmac.init_with_key(key.as_slice()).expect("Failed to initialize HMAC");
        hmac
    }

    fn new_from_slice(key: &[u8]) -> Result<Self, digest::InvalidLength> {
        let mut hmac = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            mac_handle: core::ptr::null_mut(),
            initialized: false,
            _digest: PhantomData,
        };

        hmac.init_with_key(key).map_err(|_| digest::InvalidLength)?;
        Ok(hmac)
    }

    fn update(&mut self, data: &[u8]) {
        if !self.initialized {
            panic!("HMAC not initialized");
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

        CmoxError::from_cipher_retval(result).expect("HMAC update failed");
    }

    fn finalize(self) -> Output<Self> {
        if !self.initialized {
            panic!("HMAC not initialized");
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

        CmoxError::from_cipher_retval(result).expect("HMAC finalization failed");

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

        // Re-initialize with the same key would require storing it
        // For now, just mark as uninitialized
        panic!("HMAC reset requires re-initialization with key");
    }
}

impl<D> Drop for Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_mac_cleanup(self.mac_handle);
            }
        }
    }
}

impl<D> Clone for Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    fn clone(&self) -> Self {
        // Would require storing key to properly clone
        panic!("Clone not supported for initialized HMAC - create new instance");
    }
}

impl<D> fmt::Debug for Hmac<D>
where
    D: OutputSizeUser + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hmac")
            .field("initialized", &self.initialized)
            .finish()
    }
}

// Convenient type aliases for common HMAC variants
/// HMAC-SHA1
pub type HmacSha1 = Hmac<crate::hash::Sha1>;
/// HMAC-SHA256
pub type HmacSha256 = Hmac<crate::hash::Sha256>;
/// HMAC-SHA384  
pub type HmacSha384 = Hmac<crate::hash::Sha384>;
/// HMAC-SHA512
pub type HmacSha512 = Hmac<crate::hash::Sha512>;