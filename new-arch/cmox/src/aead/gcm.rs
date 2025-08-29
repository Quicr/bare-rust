//! AES-GCM AEAD implementations using CMOX library
//!
//! This module provides full AES-GCM (Galois/Counter Mode) AEAD cipher implementations
//! using the STM32 CMOX cryptographic library. Both AES-128-GCM and AES-256-GCM are supported
//! with complete encryption, decryption, and authentication tag generation/verification.

use crate::ensure_initialized;
use crate::error::{CipherResult, FromRetval};
use aead::{
    consts::{U12, U16, U24, U32},
    AeadCore, AeadInPlace, Key, KeyInit, KeySizeUser, Nonce, Tag,
};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;

/// AES-128-GCM cipher
pub struct Aes128Gcm {
    handle: cmox_gcmFast_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 16], // Store key for reinitialization
}

/// AES-192-GCM cipher
pub struct Aes192Gcm {
    handle: cmox_gcmFast_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 24], // Store key for reinitialization
}

/// AES-256-GCM cipher
pub struct Aes256Gcm {
    handle: cmox_gcmFast_handle_t,
    cipher_handle: *mut cmox_cipher_handle_t,
    initialized: bool,
    key: [u8; 32], // Store key for reinitialization
}

// Helper macro to implement AES-GCM variants
macro_rules! impl_aes_gcm {
    ($name:ident, $key_size:ty, $key_len:expr) => {
        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl AeadCore for $name {
            type NonceSize = U12; // GCM standard nonce size is 96 bits
            type TagSize = U16; // GCM standard tag size is 128 bits
            type CiphertextOverhead = U16;
        }

        impl KeyInit for $name {
            fn new(key: &Key<Self>) -> Self {
                Self::new_with_key(key).expect("Failed to initialize AES-GCM cipher")
            }
        }

        impl $name {
            /// Create a new AES-GCM cipher with the given key
            pub fn new_with_key(key: &[u8]) -> crate::Result<Self> {
                let mut cipher = Self {
                    handle: unsafe { MaybeUninit::zeroed().assume_init() },
                    cipher_handle: core::ptr::null_mut(),
                    initialized: false,
                    key: {
                        let mut k = [0u8; $key_len];
                        let len = core::cmp::min(key.len(), $key_len);
                        k[..len].copy_from_slice(&key[..len]);
                        k
                    },
                };

                cipher.init_with_stored_key()?;
                Ok(cipher)
            }

            fn init_with_stored_key(&mut self) -> crate::Result<()> {
                ensure_initialized()?;

                // Initialize GCM handle for encryption by default
                self.cipher_handle = unsafe {
                    cmox_gcmFast_construct(&mut self.handle as *mut _, CMOX_AESFAST_GCMFAST_ENC)
                };

                if self.cipher_handle.is_null() {
                    return Err(crate::error::CipherError::Internal.into());
                }

                // Initialize cipher
                let result = unsafe { cmox_cipher_init(self.cipher_handle) };
                CipherResult::from_rv(result)?;

                // Set key
                let result = unsafe {
                    cmox_cipher_setKey(self.cipher_handle, self.key.as_ptr(), self.key.len())
                };
                CipherResult::from_rv(result)?;

                self.initialized = true;
                Ok(())
            }

            /// Encrypt data with associated data using AES-GCM
            ///
            /// This method provides the complete GCM encryption process using CMOX:
            /// - Encrypts the buffer in-place using AES-GCM
            /// - Authenticates both the encrypted data and associated data
            /// - Returns the authentication tag for verification during decryption
            pub fn encrypt_inplace(
                &mut self,
                nonce: &[u8; 12],
                associated_data: &[u8],
                buffer: &mut [u8],
            ) -> crate::Result<[u8; 16]> {
                if !self.initialized {
                    return Err(crate::error::CoreError::InitFail.into());
                }

                // Set nonce/IV
                let result =
                    unsafe { cmox_cipher_setIV(self.cipher_handle, nonce.as_ptr(), nonce.len()) };
                CipherResult::from_rv(result)?;

                // Set payload length
                let result = unsafe { cmox_cipher_setPayloadLen(self.cipher_handle, buffer.len()) };
                CipherResult::from_rv(result)?;

                // Set AAD length
                let result =
                    unsafe { cmox_cipher_setADLen(self.cipher_handle, associated_data.len()) };
                CipherResult::from_rv(result)?;

                // Set tag length
                let result = unsafe {
                    cmox_cipher_setTagLen(self.cipher_handle, 16) // 128-bit tag
                };
                CipherResult::from_rv(result)?;

                // Process AAD
                if !associated_data.is_empty() {
                    let result = unsafe {
                        cmox_cipher_appendAD(
                            self.cipher_handle,
                            associated_data.as_ptr(),
                            associated_data.len(),
                        )
                    };
                    CipherResult::from_rv(result)?;
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
                CipherResult::from_rv(result)?;

                // Generate authentication tag
                let mut tag = [0u8; 16];
                let mut tag_len = 16;
                let result = unsafe {
                    cmox_cipher_generateTag(self.cipher_handle, tag.as_mut_ptr(), &mut tag_len)
                };
                CipherResult::from_rv(result)?;

                Ok(tag)
            }

            /// Decrypt and verify data with associated data using AES-GCM
            ///
            /// This method provides the complete GCM decryption and verification process using CMOX:
            /// - Verifies the authentication tag against the encrypted data and associated data
            /// - Decrypts the buffer in-place using AES-GCM
            /// - Returns an error if authentication fails, ensuring data integrity
            pub fn decrypt_inplace(
                &mut self,
                nonce: &[u8; 12],
                associated_data: &[u8],
                buffer: &mut [u8],
                tag: &[u8; 16],
            ) -> crate::Result<()> {
                if !self.initialized {
                    return Err(crate::error::CoreError::InitFail.into());
                }

                // For decryption, we need to reinitialize with decryption implementation
                // Clean up current handle
                unsafe {
                    cmox_cipher_cleanup(self.cipher_handle);
                }

                // Construct GCM handle for decryption
                self.cipher_handle = unsafe {
                    cmox_gcmFast_construct(&mut self.handle as *mut _, CMOX_AESFAST_GCMFAST_DEC)
                };

                if self.cipher_handle.is_null() {
                    return Err(crate::error::CipherError::Internal.into());
                }

                // Initialize cipher
                let result = unsafe { cmox_cipher_init(self.cipher_handle) };
                CipherResult::from_rv(result)?;

                // Set key
                let result = unsafe {
                    cmox_cipher_setKey(self.cipher_handle, self.key.as_ptr(), self.key.len())
                };
                CipherResult::from_rv(result)?;

                // Set nonce/IV
                let result =
                    unsafe { cmox_cipher_setIV(self.cipher_handle, nonce.as_ptr(), nonce.len()) };
                CipherResult::from_rv(result)?;

                // Set payload length
                let result = unsafe { cmox_cipher_setPayloadLen(self.cipher_handle, buffer.len()) };
                CipherResult::from_rv(result)?;

                // Set AAD length
                let result =
                    unsafe { cmox_cipher_setADLen(self.cipher_handle, associated_data.len()) };
                CipherResult::from_rv(result)?;

                // Set tag length
                let result = unsafe { cmox_cipher_setTagLen(self.cipher_handle, tag.len()) };
                CipherResult::from_rv(result)?;

                // Process AAD
                if !associated_data.is_empty() {
                    let result = unsafe {
                        cmox_cipher_appendAD(
                            self.cipher_handle,
                            associated_data.as_ptr(),
                            associated_data.len(),
                        )
                    };
                    CipherResult::from_rv(result)?;
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
                CipherResult::from_rv(result)?;

                // Verify authentication tag
                let mut tag_len = tag.len() as u32;
                let result = unsafe {
                    cmox_cipher_verifyTag(
                        self.cipher_handle,
                        tag.as_ptr(),
                        &mut tag_len as *mut u32,
                    )
                };
                CipherResult::from_rv(result)?;

                // Switch back to encryption mode for future operations
                unsafe {
                    cmox_cipher_cleanup(self.cipher_handle);
                }
                self.init_with_stored_key()
            }

            /// Encrypt and authenticate data with additional authenticated data (AAD)
            pub fn encrypt_in_place_detached_mut(
                &mut self,
                nonce: &Nonce<Self>,
                aad: &[u8],
                buffer: &mut [u8],
            ) -> crate::Result<Tag<Self>> {
                if !self.initialized {
                    return Err(crate::error::CoreError::InitFail.into());
                }

                // Set nonce/IV
                let result =
                    unsafe { cmox_cipher_setIV(self.cipher_handle, nonce.as_ptr(), nonce.len()) };
                CipherResult::from_rv(result)?;

                // Set payload length
                let result = unsafe { cmox_cipher_setPayloadLen(self.cipher_handle, buffer.len()) };
                CipherResult::from_rv(result)?;

                // Set AAD length
                let result = unsafe { cmox_cipher_setADLen(self.cipher_handle, aad.len()) };
                CipherResult::from_rv(result)?;

                // Set tag length
                let result = unsafe {
                    cmox_cipher_setTagLen(self.cipher_handle, 16) // 128-bit tag
                };
                CipherResult::from_rv(result)?;

                // Process AAD
                if !aad.is_empty() {
                    let result = unsafe {
                        cmox_cipher_appendAD(self.cipher_handle, aad.as_ptr(), aad.len())
                    };
                    CipherResult::from_rv(result)?;
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
                CipherResult::from_rv(result)?;

                // Generate authentication tag
                let mut tag = Tag::<Self>::default();
                let mut tag_len = tag.len();
                let result = unsafe {
                    cmox_cipher_generateTag(self.cipher_handle, tag.as_mut_ptr(), &mut tag_len)
                };
                CipherResult::from_rv(result)?;

                Ok(tag)
            }

            /// Decrypt and verify data with additional authenticated data (AAD)
            pub fn decrypt_in_place_detached_mut(
                &mut self,
                nonce: &Nonce<Self>,
                aad: &[u8],
                buffer: &mut [u8],
                tag: &Tag<Self>,
            ) -> crate::Result<()> {
                if !self.initialized {
                    return Err(crate::error::CoreError::InitFail.into());
                }

                // For decryption, we need to reinitialize with decryption implementation
                // Clean up current handle
                unsafe {
                    cmox_cipher_cleanup(self.cipher_handle);
                }

                // Construct GCM handle for decryption
                self.cipher_handle = unsafe {
                    cmox_gcmFast_construct(&mut self.handle as *mut _, CMOX_AESFAST_GCMFAST_DEC)
                };

                if self.cipher_handle.is_null() {
                    return Err(crate::error::CipherError::Internal.into());
                }

                // Initialize cipher
                let result = unsafe { cmox_cipher_init(self.cipher_handle) };
                CipherResult::from_rv(result)?;

                // Set key
                let result = unsafe {
                    cmox_cipher_setKey(self.cipher_handle, self.key.as_ptr(), self.key.len())
                };
                CipherResult::from_rv(result)?;

                // Set nonce/IV
                let result =
                    unsafe { cmox_cipher_setIV(self.cipher_handle, nonce.as_ptr(), nonce.len()) };
                CipherResult::from_rv(result)?;

                // Set payload length
                let result = unsafe { cmox_cipher_setPayloadLen(self.cipher_handle, buffer.len()) };
                CipherResult::from_rv(result)?;

                // Set AAD length
                let result = unsafe { cmox_cipher_setADLen(self.cipher_handle, aad.len()) };
                CipherResult::from_rv(result)?;

                // Set tag length
                let result = unsafe { cmox_cipher_setTagLen(self.cipher_handle, tag.len()) };
                CipherResult::from_rv(result)?;

                // Process AAD
                if !aad.is_empty() {
                    let result = unsafe {
                        cmox_cipher_appendAD(self.cipher_handle, aad.as_ptr(), aad.len())
                    };
                    CipherResult::from_rv(result)?;
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
                CipherResult::from_rv(result)?;

                // Verify authentication tag
                let mut tag_len = tag.len() as u32;
                let result = unsafe {
                    cmox_cipher_verifyTag(
                        self.cipher_handle,
                        tag.as_ptr(),
                        &mut tag_len as *mut u32,
                    )
                };
                CipherResult::from_rv(result)?;

                // Switch back to encryption mode for future operations
                unsafe {
                    cmox_cipher_cleanup(self.cipher_handle);
                }
                self.init_with_stored_key()
            }
        }

        impl AeadInPlace for $name {
            fn encrypt_in_place_detached(
                &self,
                nonce: &Nonce<Self>,
                associated_data: &[u8],
                buffer: &mut [u8],
            ) -> aead::Result<Tag<Self>> {
                // Create a mutable copy for CMOX operations which require mutable state
                let mut cipher = self.clone();
                cipher
                    .encrypt_in_place_detached_mut(nonce, associated_data, buffer)
                    .map_err(|_| aead::Error)
            }

            fn decrypt_in_place_detached(
                &self,
                nonce: &Nonce<Self>,
                associated_data: &[u8],
                buffer: &mut [u8],
                tag: &Tag<Self>,
            ) -> aead::Result<()> {
                // Create a mutable copy for CMOX operations which require mutable state
                let mut cipher = self.clone();
                cipher
                    .decrypt_in_place_detached_mut(nonce, associated_data, buffer, tag)
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
                Self::new_with_key(&self.key).expect("Clone failed")
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

// Implement all AES-GCM variants
impl_aes_gcm!(Aes128Gcm, U16, 16);
impl_aes_gcm!(Aes192Gcm, U24, 24);
impl_aes_gcm!(Aes256Gcm, U32, 32);
