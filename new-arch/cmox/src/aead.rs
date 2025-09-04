//! CMOX-based AEAD ciphers
//!
//! This module provides access to all of the hash functions exposed by CMOX:
//!
//! * SHA-1
//! * SHA-224, SHA-256, SHA-384, SHA-512, SHA-512_224, SHA512_256
//! * SHA3-224, SHA3-256, SHA3-384, SHA3-512, SHAKE-128, SHAKE-256
//!
//! Each algorithm is implemented by a struct of the same name, which implements the Rust Crypto
//! Digest trait.
//!
//! SHAKE128 and SHAKE256 are not implemented, because the Rust Crypto XofReader trait doesn't seem
//! like it aligns well with the CMOX API.
#![allow(missing_docs)]

use crate::ensure_initialized;
use crate::error::{CipherResult, FromRetval};
use aead::{
    consts::{U12, U16, U32},
    generic_array::{typenum::Unsigned, ArrayLength},
    AeadCore, AeadInPlace, Key, KeyInit, KeySizeUser, Nonce, Tag,
};
use cmox_sys::*;
use core::mem::MaybeUninit;

pub trait CipherType {
    type KeySize: ArrayLength<u8>;
    type NonceSize: ArrayLength<u8>;
    type TagSize: ArrayLength<u8>;
    type RawHandle;
    fn construct_enc(raw_handle: &mut Self::RawHandle) -> *mut cmox_cipher_handle_t;
    fn construct_dec(raw_handle: &mut Self::RawHandle) -> *mut cmox_cipher_handle_t;
}

macro_rules! cipher {
    ($cipher:ident, $type:ident, $key_size:ty, $nonce_size:ty, $tag_size:ty, $handle:ty, $construct:ident, $enc_param:ident, $dec_param:ident) => {
        pub type $cipher = CipherImpl<$type>;

        pub struct $type;

        impl CipherType for $type {
            type KeySize = $key_size;
            type NonceSize = $nonce_size;
            type TagSize = $tag_size;
            type RawHandle = $handle;

            fn construct_enc(raw_handle: &mut Self::RawHandle) -> *mut cmox_cipher_handle_t {
                unsafe { $construct(raw_handle as *mut _, $enc_param) }
            }

            fn construct_dec(raw_handle: &mut Self::RawHandle) -> *mut cmox_cipher_handle_t {
                unsafe { $construct(raw_handle as *mut _, $dec_param) }
            }
        }
    };
}

// AES-128-GCM with AES=Fast, GCM=Fast
cipher! {
    Aes128FastGcmFast, Aes128FastGcmFastType,
    U16, U12, U16,
    cmox_gcmFast_handle_t, cmox_gcmFast_construct,
    CMOX_AESFAST_GCMFAST_ENC, CMOX_AESFAST_GCMFAST_DEC
}

// AES-128-GCM with AES=Small, GCM=Fast
cipher! {
    Aes128SmallGcmFast, Aes128SmallGcmFastType,
    U16, U12, U16,
    cmox_gcmFast_handle_t, cmox_gcmFast_construct,
    CMOX_AESSMALL_GCMFAST_ENC, CMOX_AESSMALL_GCMFAST_DEC
}

// AES-128-GCM with AES=Fast, GCM=Small
cipher! {
    Aes128FastGcmSmall, Aes128FastGcmSmallType,
    U16, U12, U16,
    cmox_gcmSmall_handle_t, cmox_gcmSmall_construct,
    CMOX_AESFAST_GCMSMALL_ENC, CMOX_AESFAST_GCMSMALL_DEC
}

// AES-128-GCM with AES=Small, GCM=Small
cipher! {
    Aes128SmallGcmSmall, Aes128SmallGcmSmallType,
    U16, U12, U16,
    cmox_gcmSmall_handle_t, cmox_gcmSmall_construct,
    CMOX_AESSMALL_GCMSMALL_ENC, CMOX_AESSMALL_GCMSMALL_DEC
}

// AES-256-GCM with AES=Fast, GCM=Fast
cipher! {
    Aes256FastGcmFast, Aes256FastGcmFastType,
    U32, U12, U16,
    cmox_gcmFast_handle_t, cmox_gcmFast_construct,
    CMOX_AESFAST_GCMFAST_ENC, CMOX_AESFAST_GCMFAST_DEC
}

// AES-256-GCM with AES=Small, GCM=Fast
cipher! {
    Aes256SmallGcmFast, Aes256SmallGcmFastType,
    U32, U12, U16,
    cmox_gcmFast_handle_t, cmox_gcmFast_construct,
    CMOX_AESSMALL_GCMFAST_ENC, CMOX_AESSMALL_GCMFAST_DEC
}

// AES-256-GCM with AES=Fast, GCM=Small
cipher! {
    Aes256FastGcmSmall, Aes256FastGcmSmallType,
    U32, U12, U16,
    cmox_gcmSmall_handle_t, cmox_gcmSmall_construct,
    CMOX_AESFAST_GCMSMALL_ENC, CMOX_AESFAST_GCMSMALL_DEC
}

// AES-256-GCM with AES=Small, GCM=Small
cipher! {
    Aes256SmallGcmSmall, Aes256SmallGcmSmallType,
    U32, U12, U16,
    cmox_gcmSmall_handle_t, cmox_gcmSmall_construct,
    CMOX_AESSMALL_GCMSMALL_ENC, CMOX_AESSMALL_GCMSMALL_DEC
}

// AES-128-CCM with AES=Fast
cipher! {
    Aes128FastCcm, Aes128FastCcmType,
    U16, U12, U16,
    cmox_ccm_handle_t, cmox_ccm_construct,
    CMOX_AESFAST_CCM_ENC, CMOX_AESFAST_CCM_DEC
}

// AES-128-CCM with AES=Small
cipher! {
    Aes128SmallCcm, Aes128SmallCcmType,
    U16, U12, U16,
    cmox_ccm_handle_t, cmox_ccm_construct,
    CMOX_AESSMALL_CCM_ENC, CMOX_AESSMALL_CCM_DEC
}

// AES-256-CCM with AES=Fast
cipher! {
    Aes256FastCcm, Aes256FastCcmType,
    U32, U12, U16,
    cmox_ccm_handle_t, cmox_ccm_construct,
    CMOX_AESFAST_CCM_ENC, CMOX_AESFAST_CCM_DEC
}

// AES-256-CCM with AES=Small
cipher! {
    Aes256SmallCcm, Aes256SmallCcmType,
    U32, U12, U16,
    cmox_ccm_handle_t, cmox_ccm_construct,
    CMOX_AESSMALL_CCM_ENC, CMOX_AESSMALL_CCM_DEC
}

// ChaChaPoly
cipher! {
    ChaChaPoly, ChaChaPolyType,
    U32, U12, U16,
    cmox_chachapoly_handle_t, cmox_chachapoly_construct,
    CMOX_CHACHAPOLY_ENC, CMOX_CHACHAPOLY_DEC
}

pub struct CipherImpl<C: CipherType> {
    raw_enc_handle: C::RawHandle,
    raw_dec_handle: C::RawHandle,
    enc_handle: *mut cmox_cipher_handle_t,
    dec_handle: *mut cmox_cipher_handle_t,
}

impl<C: CipherType> CipherImpl<C> {
    fn new(key: &[u8]) -> Self {
        ensure_initialized().expect("CMOX library not initialized");

        let mut raw_enc_handle: C::RawHandle = unsafe { MaybeUninit::zeroed().assume_init() };
        let enc_handle = C::construct_enc(&mut raw_enc_handle);

        let mut raw_dec_handle: C::RawHandle = unsafe { MaybeUninit::zeroed().assume_init() };
        let dec_handle = C::construct_dec(&mut raw_dec_handle);

        if enc_handle.is_null() || dec_handle.is_null() {
            panic!("Failed to construct MAC handle");
        }

        unsafe {
            CipherResult::from_rv(cmox_cipher_init(enc_handle)).expect("Cipher init failed");
            CipherResult::from_rv(cmox_cipher_setKey(enc_handle, key.as_ptr(), key.len()))
                .expect("Cipher set key failed");

            CipherResult::from_rv(cmox_cipher_init(dec_handle)).expect("Cipher init failed");
            CipherResult::from_rv(cmox_cipher_setKey(dec_handle, key.as_ptr(), key.len()))
                .expect("Cipher set key failed");
        }

        Self {
            raw_enc_handle,
            raw_dec_handle,
            enc_handle,
            dec_handle,
        }
    }

    unsafe fn setup(
        handle: *mut cmox_cipher_handle_t,
        nonce: &[u8],
        aad: &[u8],
        buffer: &[u8],
    ) -> CipherResult {
        CipherResult::from_rv(cmox_cipher_setIV(handle, nonce.as_ptr(), nonce.len()))?;
        CipherResult::from_rv(cmox_cipher_setPayloadLen(handle, buffer.len()))?;
        CipherResult::from_rv(cmox_cipher_setADLen(handle, aad.len()))?;
        CipherResult::from_rv(cmox_cipher_setTagLen(handle, C::TagSize::USIZE))?;

        if !aad.is_empty() {
            CipherResult::from_rv(cmox_cipher_appendAD(handle, aad.as_ptr(), aad.len()))?
        }

        Ok(())
    }

    fn seal(&self, nonce: &[u8], aad: &[u8], buffer: &mut [u8], tag: &mut [u8]) -> CipherResult {
        unsafe {
            Self::setup(self.enc_handle, nonce, aad, buffer)?;
        }

        // Encrypt payload
        let mut output_len = buffer.len();
        unsafe {
            CipherResult::from_rv(cmox_cipher_append(
                self.enc_handle,
                buffer.as_ptr(),
                buffer.len(),
                buffer.as_mut_ptr(),
                &mut output_len,
            ))?;
        }

        // Generate authentication tag
        let mut tag_len = 16;
        unsafe {
            CipherResult::from_rv(cmox_cipher_generateTag(
                self.enc_handle,
                tag.as_mut_ptr(),
                &mut tag_len,
            ))?
        };

        Ok(())
    }

    fn open(&self, nonce: &[u8], aad: &[u8], buffer: &mut [u8], tag: &[u8]) -> CipherResult {
        unsafe {
            Self::setup(self.dec_handle, nonce, aad, buffer)?;
        }

        // Encrypt payload
        let mut output_len = buffer.len();
        unsafe {
            CipherResult::from_rv(cmox_cipher_append(
                self.dec_handle,
                buffer.as_ptr(),
                buffer.len(),
                buffer.as_mut_ptr(),
                &mut output_len,
            ))?;
        }

        // Generate authentication tag
        let mut tag_len = 16;
        unsafe {
            CipherResult::from_rv(cmox_cipher_verifyTag(
                self.dec_handle,
                tag.as_ptr(),
                &mut tag_len,
            ))?
        };

        Ok(())
    }

    fn cleanup(&mut self) {
        unsafe {
            cmox_cipher_cleanup(self.enc_handle);
            cmox_cipher_cleanup(self.dec_handle);
        }
    }
}

impl<C: CipherType> KeySizeUser for CipherImpl<C> {
    type KeySize = C::KeySize;
}

impl<C: CipherType> AeadCore for CipherImpl<C> {
    type NonceSize = C::NonceSize;
    type TagSize = C::TagSize;
    type CiphertextOverhead = C::TagSize;
}

impl<C: CipherType> KeyInit for CipherImpl<C> {
    fn new(key: &Key<Self>) -> Self {
        Self::new(key.as_slice())
    }
}

impl<C: CipherType> AeadInPlace for CipherImpl<C> {
    fn encrypt_in_place_detached(
        &self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
    ) -> aead::Result<Tag<Self>> {
        let mut tag: Tag<Self> = Default::default();
        self.seal(nonce, associated_data, buffer, &mut tag)
            .map_err(|_| aead::Error)?;
        Ok(tag)
    }

    fn decrypt_in_place_detached(
        &self,
        nonce: &Nonce<Self>,
        associated_data: &[u8],
        buffer: &mut [u8],
        tag: &Tag<Self>,
    ) -> aead::Result<()> {
        self.open(nonce, associated_data, buffer, tag)
            .map_err(|_| aead::Error)?;
        Ok(())
    }
}
impl<C: CipherType> Drop for CipherImpl<C> {
    fn drop(&mut self) {
        self.cleanup();
    }
}
