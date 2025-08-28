//! Message Authentication Code (MAC) implementations

use crate::{utils::ensure_initialized, CmoxError, Result};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::{
    generic_array::GenericArray,
    CtOutput, MacError, MacMarker, OutputSizeUser,
    Update, Key, Mac, KeyInit,
    core_api::BlockSizeUser,
};
use cipher::KeySizeUser;

/// HMAC using SHA-256
pub struct HmacSha256 {
    handle: cmox_hmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
}

impl MacMarker for HmacSha256 {}

impl OutputSizeUser for HmacSha256 {
    type OutputSize = digest::consts::U32; // SHA-256 produces 256-bit output
}

impl KeySizeUser for HmacSha256 {
    type KeySize = digest::consts::U32; // SHA-256 key size (can be any size but we use 32)
}

impl BlockSizeUser for HmacSha256 {
    type BlockSize = digest::consts::U64; // SHA-256 block size
}

impl Update for HmacSha256 {
    fn update(&mut self, data: &[u8]) {
        self.update_internal(data).expect("HMAC update failed");
    }
}

impl KeyInit for HmacSha256 {
    fn new(key: &Key<Self>) -> Self {
        Self::new_with_key(key.as_slice()).expect("Failed to initialize HMAC-SHA256")
    }

    fn new_from_slice(key: &[u8]) -> core::result::Result<Self, digest::InvalidLength> {
        Self::new_with_key(key).map_err(|_| digest::InvalidLength)
    }
}

impl HmacSha256 {
    /// Create a new HMAC-SHA256 instance with the given key
    pub fn new_with_key(key: &[u8]) -> Result<Self> {
        let mut hmac = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            mac_handle: core::ptr::null_mut(),
            initialized: false,
        };

        hmac.init_with_key(key)?;
        Ok(hmac)
    }

    fn init_with_key(&mut self, key: &[u8]) -> Result<()> {
        ensure_initialized()?;

        // Initialize HMAC handle with SHA-256
        self.mac_handle = unsafe {
            cmox_hmac_construct(
                &mut self.handle as *mut _,
                CMOX_HMAC_SHA256,
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

    /// Update the MAC with input data
    pub fn update_internal(&mut self, data: &[u8]) -> Result<()> {
        if !self.initialized {
            return Err(CmoxError::NotInitialized);
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
    pub fn finalize_internal(self) -> Result<[u8; 32]> {
        if !self.initialized {
            return Err(CmoxError::NotInitialized);
        }

        let mut tag = [0u8; 32];
        let mut tag_len = tag.len();

        let result = unsafe {
            cmox_mac_generateTag(
                self.mac_handle,
                tag.as_mut_ptr(),
                &mut tag_len,
            )
        };

        CmoxError::from_cipher_retval(result)?;

        // Clean up
        unsafe {
            cmox_mac_cleanup(self.mac_handle);
        }

        Ok(tag)
    }
}

impl Mac for HmacSha256 {
    fn new(key: &Key<Self>) -> Self {
        Self::new_with_key(key.as_slice()).expect("Failed to initialize HMAC-SHA256")
    }

    fn new_from_slice(key: &[u8]) -> core::result::Result<Self, digest::InvalidLength> {
        Self::new_with_key(key).map_err(|_| digest::InvalidLength)
    }

    fn update(&mut self, data: &[u8]) {
        self.update_internal(data).expect("HMAC update failed");
    }

    fn chain_update(mut self, data: impl AsRef<[u8]>) -> Self {
        self.update_internal(data.as_ref()).expect("HMAC update failed");
        self
    }

    fn finalize(self) -> CtOutput<Self> {
        let result = self.finalize_internal().expect("HMAC finalization failed");
        CtOutput::new(GenericArray::clone_from_slice(&result))
    }

    fn finalize_reset(&mut self) -> CtOutput<Self> {
        // Not supported by CMOX API - would need to store key and reinitialize
        unimplemented!("finalize_reset not supported by CMOX API")
    }

    fn reset(&mut self) {
        if self.initialized {
            unsafe {
                cmox_mac_cleanup(self.mac_handle);
            }
            self.initialized = false;
        }
        // Note: Reset would require re-initialization with key
        panic!("HMAC reset requires re-initialization with key");
    }

    fn verify(self, tag: &GenericArray<u8, Self::OutputSize>) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        if computed.into_bytes().eq(tag) {
            Ok(())
        } else {
            Err(MacError)
        }
    }

    fn verify_reset(
        &mut self,
        _tag: &GenericArray<u8, Self::OutputSize>,
    ) -> core::result::Result<(), MacError> {
        unimplemented!("verify_reset not supported")
    }

    fn verify_slice(self, tag: &[u8]) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        if computed.into_bytes().as_slice().eq(tag) {
            Ok(())
        } else {
            Err(MacError)
        }
    }

    fn verify_slice_reset(&mut self, _tag: &[u8]) -> core::result::Result<(), MacError> {
        unimplemented!("verify_slice_reset not supported")
    }

    fn verify_truncated_left(self, tag: &[u8]) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        let computed_bytes = computed.into_bytes();
        if tag.len() <= computed_bytes.len() && computed_bytes[..tag.len()].eq(tag) {
            Ok(())
        } else {
            Err(MacError)
        }
    }

    fn verify_truncated_right(self, tag: &[u8]) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        let computed_bytes = computed.into_bytes();
        if tag.len() <= computed_bytes.len() 
            && computed_bytes[(computed_bytes.len() - tag.len())..].eq(tag) 
        {
            Ok(())
        } else {
            Err(MacError)
        }
    }
}

impl Drop for HmacSha256 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_mac_cleanup(self.mac_handle);
            }
        }
    }
}

impl Clone for HmacSha256 {
    fn clone(&self) -> Self {
        panic!("Clone not supported for initialized HMAC - create new instance");
    }
}

impl fmt::Debug for HmacSha256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HmacSha256")
            .field("initialized", &self.initialized)
            .finish()
    }
}

/// AES-CMAC implementation
pub struct AesCmac {
    handle: cmox_cmac_handle_t,
    mac_handle: *mut cmox_mac_handle_t,
    initialized: bool,
}

impl MacMarker for AesCmac {}

impl OutputSizeUser for AesCmac {
    type OutputSize = digest::consts::U16; // AES-CMAC produces 128-bit output
}

impl KeySizeUser for AesCmac {
    type KeySize = digest::consts::U16; // AES key size (16 bytes for AES-128)
}

impl BlockSizeUser for AesCmac {
    type BlockSize = digest::consts::U16; // AES block size is 128 bits
}

impl Update for AesCmac {
    fn update(&mut self, data: &[u8]) {
        self.update_internal(data).expect("CMAC update failed");
    }
}

impl KeyInit for AesCmac {
    fn new(key: &Key<Self>) -> Self {
        Self::new_with_key(key.as_slice()).expect("Failed to initialize AES-CMAC")
    }

    fn new_from_slice(key: &[u8]) -> core::result::Result<Self, digest::InvalidLength> {
        Self::new_with_key(key).map_err(|_| digest::InvalidLength)
    }
}

impl AesCmac {
    /// Create a new AES-CMAC instance with the given key
    pub fn new_with_key(key: &[u8]) -> Result<Self> {
        let mut cmac = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            mac_handle: core::ptr::null_mut(),
            initialized: false,
        };

        cmac.init_with_key(key)?;
        Ok(cmac)
    }

    fn init_with_key(&mut self, key: &[u8]) -> Result<()> {
        ensure_initialized()?;

        // Initialize CMAC handle with AES
        self.mac_handle = unsafe {
            cmox_cmac_construct(
                &mut self.handle as *mut _,
                CMOX_CMAC_AESFAST,
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

    /// Update the MAC with input data
    pub fn update_internal(&mut self, data: &[u8]) -> Result<()> {
        if !self.initialized {
            return Err(CmoxError::NotInitialized);
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
    pub fn finalize_internal(self) -> Result<[u8; 16]> {
        if !self.initialized {
            return Err(CmoxError::NotInitialized);
        }

        let mut tag = [0u8; 16];
        let mut tag_len = tag.len();

        let result = unsafe {
            cmox_mac_generateTag(
                self.mac_handle,
                tag.as_mut_ptr(),
                &mut tag_len,
            )
        };

        CmoxError::from_cipher_retval(result)?;

        // Clean up
        unsafe {
            cmox_mac_cleanup(self.mac_handle);
        }

        Ok(tag)
    }
}

impl Mac for AesCmac {
    fn new(key: &Key<Self>) -> Self {
        Self::new_with_key(key.as_slice()).expect("Failed to initialize AES-CMAC")
    }

    fn new_from_slice(key: &[u8]) -> core::result::Result<Self, digest::InvalidLength> {
        Self::new_with_key(key).map_err(|_| digest::InvalidLength)
    }

    fn update(&mut self, data: &[u8]) {
        self.update_internal(data).expect("CMAC update failed");
    }

    fn chain_update(mut self, data: impl AsRef<[u8]>) -> Self {
        self.update_internal(data.as_ref()).expect("CMAC update failed");
        self
    }

    fn finalize(self) -> CtOutput<Self> {
        let result = self.finalize_internal().expect("CMAC finalization failed");
        CtOutput::new(GenericArray::clone_from_slice(&result))
    }

    fn finalize_reset(&mut self) -> CtOutput<Self> {
        unimplemented!("finalize_reset not supported")
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

    fn verify(self, tag: &GenericArray<u8, Self::OutputSize>) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        if computed.into_bytes().eq(tag) {
            Ok(())
        } else {
            Err(MacError)
        }
    }

    fn verify_reset(
        &mut self,
        _tag: &GenericArray<u8, Self::OutputSize>,
    ) -> core::result::Result<(), MacError> {
        unimplemented!("verify_reset not supported")
    }

    fn verify_slice(self, tag: &[u8]) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        if computed.into_bytes().as_slice().eq(tag) {
            Ok(())
        } else {
            Err(MacError)
        }
    }

    fn verify_slice_reset(&mut self, _tag: &[u8]) -> core::result::Result<(), MacError> {
        unimplemented!("verify_slice_reset not supported")
    }

    fn verify_truncated_left(self, tag: &[u8]) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        let computed_bytes = computed.into_bytes();
        if tag.len() <= computed_bytes.len() && computed_bytes[..tag.len()].eq(tag) {
            Ok(())
        } else {
            Err(MacError)
        }
    }

    fn verify_truncated_right(self, tag: &[u8]) -> core::result::Result<(), MacError> {
        let computed = self.finalize();
        let computed_bytes = computed.into_bytes();
        if tag.len() <= computed_bytes.len() 
            && computed_bytes[(computed_bytes.len() - tag.len())..].eq(tag) 
        {
            Ok(())
        } else {
            Err(MacError)
        }
    }
}

impl Drop for AesCmac {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_mac_cleanup(self.mac_handle);
            }
        }
    }
}

impl Clone for AesCmac {
    fn clone(&self) -> Self {
        panic!("Clone not supported for initialized CMAC - create new instance");
    }
}

impl fmt::Debug for AesCmac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AesCmac")
            .field("initialized", &self.initialized)
            .finish()
    }
}

// mod test; // Tests require std environment