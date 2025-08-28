//! AES-CCM AEAD implementations
//!
//! AES-CCM (Counter with CBC-MAC) provides authenticated encryption
//! with associated data (AEAD) using AES in a combination of CTR mode
//! for confidentiality and CBC-MAC for authenticity.

use crate::{utils::ensure_initialized, CmoxError, Result};
use cmox_sys::*;
use aead::{
    consts::{U12, U16, U24, U32},
    AeadCore, AeadInPlace, Key, KeyInit, KeySizeUser, Nonce, Tag,
};
use core::fmt;
use core::mem::MaybeUninit;

/// AES-128-CCM cipher
pub struct Aes128Ccm {
    handle: cmox_ccm_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 16], // Store key for reinitialization
}

/// AES-192-CCM cipher
pub struct Aes192Ccm {
    handle: cmox_ccm_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 24], // Store key for reinitialization
}

/// AES-256-CCM cipher
pub struct Aes256Ccm {
    handle: cmox_ccm_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 32], // Store key for reinitialization
}

// Helper macro to implement AES-CCM variants
macro_rules! impl_aes_ccm {
    ($name:ident, $key_size:ty, $key_len:expr) => {
        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl AeadCore for $name {
            type NonceSize = U12; // CCM standard nonce size is 96 bits
            type TagSize = U16;   // CCM standard tag size is 128 bits
            type CiphertextOverhead = U16;
        }

        impl KeyInit for $name {
            fn new(key: &Key<Self>) -> Self {
                let mut cipher = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    cipher_handle: core::ptr::null_mut(),
                    initialized: false,
                    key: [0u8; $key_len],
                };

                cipher.key[..key.len()].copy_from_slice(key.as_slice());
                cipher.init().expect("Failed to initialize AES-CCM");
                cipher
            }
        }

        impl $name {
            /// Create a new AES-CCM cipher with the given key
            pub fn new_with_key(key: &[u8]) -> Result<Self> {
                if key.len() != $key_len {
                    return Err(CmoxError::InvalidInputSize);
                }

                let mut cipher = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    cipher_handle: core::ptr::null_mut(),
                    initialized: false,
                    key: [0u8; $key_len],
                };

                cipher.key[..key.len()].copy_from_slice(key);
                cipher.init()?;
                Ok(cipher)
            }

            fn init(&mut self) -> Result<()> {
                ensure_initialized()?;

                // Construct CCM handle
                self.cipher_handle = unsafe {
                    cmox_ccm_construct(&mut self.handle as *mut _, CMOX_AESFAST_CCM_ENC)
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

                Ok(())
            }
        }

        impl AeadInPlace for $name {
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

        impl Drop for $name {
            fn drop(&mut self) {
                if self.initialized {
                    unsafe {
                        cmox_cipher_cleanup(self.cipher_handle);
                    }
                }
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                let mut new_cipher = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    cipher_handle: core::ptr::null_mut(),
                    initialized: false,
                    key: self.key,
                };
                
                new_cipher.init().expect("Failed to clone AES-CCM");
                new_cipher
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

// Implement all AES-CCM variants
impl_aes_ccm!(Aes128Ccm, U16, 16);
impl_aes_ccm!(Aes192Ccm, U24, 24);
impl_aes_ccm!(Aes256Ccm, U32, 32);