//! ChaCha20-Poly1305 AEAD implementation
//!
//! ChaCha20-Poly1305 is a modern AEAD (Authenticated Encryption with Associated Data)
//! cipher combining the ChaCha20 stream cipher for confidentiality with the Poly1305
//! authenticator for integrity and authentication.

use crate::{utils::ensure_initialized, CmoxError, Result};
use cmox_sys::*;
use aead::{
    consts::{U12, U16, U32},
    AeadCore, AeadInPlace, Key, KeyInit, KeySizeUser, Nonce, Tag,
};
use core::fmt;
use core::mem::MaybeUninit;

/// ChaCha20-Poly1305 AEAD cipher
pub struct ChaCha20Poly1305 {
    handle: cmox_chachapoly_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 32], // ChaCha20 uses 256-bit keys
}

impl KeySizeUser for ChaCha20Poly1305 {
    type KeySize = U32;
}

impl AeadCore for ChaCha20Poly1305 {
    type NonceSize = U12; // ChaCha20-Poly1305 uses 96-bit nonces
    type TagSize = U16;   // Poly1305 produces 128-bit tags
    type CiphertextOverhead = U16;
}

impl KeyInit for ChaCha20Poly1305 {
    fn new(key: &Key<Self>) -> Self {
        let mut cipher = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            cipher_handle: core::ptr::null_mut(),
            initialized: false,
            key: [0u8; 32],
        };

        cipher.key.copy_from_slice(key.as_slice());
        cipher.init().expect("Failed to initialize ChaCha20-Poly1305");
        cipher
    }
}

impl ChaCha20Poly1305 {
    /// Create a new ChaCha20-Poly1305 cipher with the given key
    pub fn new_with_key(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(CmoxError::InvalidInputSize);
        }

        let mut cipher = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            cipher_handle: core::ptr::null_mut(),
            initialized: false,
            key: [0u8; 32],
        };

        cipher.key.copy_from_slice(key);
        cipher.init()?;
        Ok(cipher)
    }

    fn init(&mut self) -> Result<()> {
        ensure_initialized()?;

        // Construct ChaCha20-Poly1305 handle
        self.cipher_handle = unsafe {
            cmox_chachapoly_construct(
                &mut self.handle as *mut _,
                CMOX_CHACHAPOLY_ENC, // Use encryption implementation
            )
        };

        if self.cipher_handle.is_null() {
            return Err(CmoxError::InitializationFailed);
        }

        // Initialize cipher
        let result = unsafe {
            cmox_cipher_init(self.cipher_handle)
        };
        CmoxError::from_cipher_retval(result)?;

        // Set key
        let result = unsafe {
            cmox_cipher_setKey(
                self.cipher_handle,
                self.key.as_ptr(),
                self.key.len(),
            )
        };
        CmoxError::from_cipher_retval(result)?;

        self.initialized = true;
        Ok(())
    }

    /// Encrypt and authenticate data with additional authenticated data (AAD)
    pub fn encrypt_in_place_detached_mut(
        &mut self,
        nonce: &Nonce<Self>,
        aad: &[u8],
        buffer: &mut [u8],
    ) -> Result<Tag<Self>> {
        if !self.initialized {
            return Err(CmoxError::NotInitialized);
        }

        // Set nonce/IV
        let result = unsafe {
            cmox_cipher_setIV(
                self.cipher_handle,
                nonce.as_ptr(),
                nonce.len(),
            )
        };
        CmoxError::from_cipher_retval(result)?;

        // Set payload length
        let result = unsafe {
            cmox_cipher_setPayloadLen(self.cipher_handle, buffer.len())
        };
        CmoxError::from_cipher_retval(result)?;

        // Set AAD length
        let result = unsafe {
            cmox_cipher_setADLen(self.cipher_handle, aad.len())
        };
        CmoxError::from_cipher_retval(result)?;

        // Set tag length
        let result = unsafe {
            cmox_cipher_setTagLen(self.cipher_handle, 16) // 128-bit tag
        };
        CmoxError::from_cipher_retval(result)?;

        // Process AAD
        if !aad.is_empty() {
            let result = unsafe {
                cmox_cipher_appendAD(
                    self.cipher_handle,
                    aad.as_ptr(),
                    aad.len(),
                )
            };
            CmoxError::from_cipher_retval(result)?;
        }

        // Encrypt payload
        let mut output_len = buffer.len();
        let result = unsafe {
            cmox_cipher_append(
                self.cipher_handle,
                buffer.as_ptr(),
                buffer.len(),
                buffer.as_mut_ptr(),
                &mut output_len,
            )
        };
        CmoxError::from_cipher_retval(result)?;

        // Generate authentication tag
        let mut tag = Tag::<Self>::default();
        let mut tag_len = tag.len();
        let result = unsafe {
            cmox_cipher_generateTag(
                self.cipher_handle,
                tag.as_mut_ptr(),
                &mut tag_len,
            )
        };
        CmoxError::from_cipher_retval(result)?;

        Ok(tag)
    }

    /// Decrypt and verify data with additional authenticated data (AAD)
    pub fn decrypt_in_place_detached_mut(
        &mut self,
        nonce: &Nonce<Self>,
        aad: &[u8],
        buffer: &mut [u8],
        tag: &Tag<Self>,
    ) -> Result<()> {
        if !self.initialized {
            return Err(CmoxError::NotInitialized);
        }

        // For decryption, we need to reinitialize with decryption implementation
        // Clean up current handle
        unsafe {
            cmox_cipher_cleanup(self.cipher_handle);
        }

        // Construct ChaCha20-Poly1305 handle for decryption
        self.cipher_handle = unsafe {
            cmox_chachapoly_construct(
                &mut self.handle as *mut _,
                CMOX_CHACHAPOLY_DEC, // Use decryption implementation
            )
        };

        if self.cipher_handle.is_null() {
            return Err(CmoxError::InitializationFailed);
        }

        // Initialize cipher
        let result = unsafe {
            cmox_cipher_init(self.cipher_handle)
        };
        CmoxError::from_cipher_retval(result)?;

        // Set key
        let result = unsafe {
            cmox_cipher_setKey(
                self.cipher_handle,
                self.key.as_ptr(),
                self.key.len(),
            )
        };
        CmoxError::from_cipher_retval(result)?;

        // Set nonce/IV
        let result = unsafe {
            cmox_cipher_setIV(
                self.cipher_handle,
                nonce.as_ptr(),
                nonce.len(),
            )
        };
        CmoxError::from_cipher_retval(result)?;

        // Set payload length
        let result = unsafe {
            cmox_cipher_setPayloadLen(self.cipher_handle, buffer.len())
        };
        CmoxError::from_cipher_retval(result)?;

        // Set AAD length
        let result = unsafe {
            cmox_cipher_setADLen(self.cipher_handle, aad.len())
        };
        CmoxError::from_cipher_retval(result)?;

        // Set tag length
        let result = unsafe {
            cmox_cipher_setTagLen(self.cipher_handle, tag.len())
        };
        CmoxError::from_cipher_retval(result)?;

        // Process AAD
        if !aad.is_empty() {
            let result = unsafe {
                cmox_cipher_appendAD(
                    self.cipher_handle,
                    aad.as_ptr(),
                    aad.len(),
                )
            };
            CmoxError::from_cipher_retval(result)?;
        }

        // Decrypt payload
        let mut output_len = buffer.len();
        let result = unsafe {
            cmox_cipher_append(
                self.cipher_handle,
                buffer.as_ptr(),
                buffer.len(),
                buffer.as_mut_ptr(),
                &mut output_len,
            )
        };
        CmoxError::from_cipher_retval(result)?;

        // Verify authentication tag
        let mut tag_len = tag.len() as u32;
        let result = unsafe {
            cmox_cipher_verifyTag(
                self.cipher_handle,
                tag.as_ptr(),
                &mut tag_len as *mut u32,
            )
        };
        CmoxError::from_cipher_retval(result)?;

        // Switch back to encryption mode for future operations
        unsafe {
            cmox_cipher_cleanup(self.cipher_handle);
        }
        self.init()?;

        Ok(())
    }
}

impl AeadInPlace for ChaCha20Poly1305 {
    fn encrypt_in_place_detached(
        &self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> aead::Result<Tag<Self>> {
        // Need to create a mutable copy since CMOX operations require mutable state
        let mut cipher = self.clone();
        cipher.encrypt_in_place_detached_mut(nonce, associated_data, buffer)
            .map_err(|_| aead::Error)
    }

    fn decrypt_in_place_detached(
        &self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &Tag<Self>,
    ) -> aead::Result<()> {
        // Need to create a mutable copy since CMOX operations require mutable state
        let mut cipher = self.clone();
        cipher.decrypt_in_place_detached_mut(nonce, associated_data, buffer, tag)
            .map_err(|_| aead::Error)
    }
}

impl Drop for ChaCha20Poly1305 {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                cmox_cipher_cleanup(self.cipher_handle);
            }
        }
    }
}

impl Clone for ChaCha20Poly1305 {
    fn clone(&self) -> Self {
        let mut new_cipher = Self {
            handle: unsafe { MaybeUninit::zeroed().assume_init() },
            cipher_handle: core::ptr::null_mut(),
            initialized: false,
            key: self.key,
        };
        
        new_cipher.init().expect("Failed to clone ChaCha20-Poly1305");
        new_cipher
    }
}

impl fmt::Debug for ChaCha20Poly1305 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChaCha20Poly1305")
            .field("initialized", &self.initialized)
            .finish()
    }
}