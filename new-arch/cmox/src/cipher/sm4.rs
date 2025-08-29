//! SM4 block cipher implementation

use crate::ensure_initialized;
use crate::error::{CipherResult, FromRetval};
use cipher::{consts::U16, Block, BlockSizeUser, Key, KeyInit, KeySizeUser};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;

/// SM4 cipher
pub struct Sm4 {
    enc_handle: cmox_ecb_handle_t,
    dec_handle: cmox_ecb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

impl KeySizeUser for Sm4 {
    type KeySize = U16;
}

impl BlockSizeUser for Sm4 {
    type BlockSize = U16;
}

impl KeyInit for Sm4 {
    fn new(key: &Key<Self>) -> Self {
        ensure_initialized().expect("CMOX library not initialized");
        
        let mut enc_handle = unsafe { MaybeUninit::zeroed().assume_init() };
        let mut dec_handle = unsafe { MaybeUninit::zeroed().assume_init() };
        
        let enc_cipher = unsafe { cmox_ecb_construct(&mut enc_handle as *mut _, CMOX_SM4_ECB_ENC) };
        let dec_cipher = unsafe { cmox_ecb_construct(&mut dec_handle as *mut _, CMOX_SM4_ECB_DEC) };

        if enc_cipher.is_null() || dec_cipher.is_null() {
            panic!("Failed to construct SM4 cipher handles");
        }

        unsafe {
            CipherResult::from_rv(cmox_cipher_init(enc_cipher))
                .expect("Failed to initialize SM4 encryption cipher");
            CipherResult::from_rv(cmox_cipher_init(dec_cipher))
                .expect("Failed to initialize SM4 decryption cipher");
            CipherResult::from_rv(cmox_cipher_setKey(enc_cipher, key.as_ptr(), key.len()))
                .expect("Failed to set SM4 encryption key");
            CipherResult::from_rv(cmox_cipher_setKey(dec_cipher, key.as_ptr(), key.len()))
                .expect("Failed to set SM4 decryption key");
        }

        Self {
            enc_handle,
            dec_handle,
            enc_cipher,
            dec_cipher,
        }
    }
}

impl Sm4 {
    /// Create a new SM4 cipher with the given key
    pub fn new_with_key(key: &[u8]) -> crate::Result<Self> {
        ensure_initialized()?;
        
        let mut enc_handle = unsafe { MaybeUninit::zeroed().assume_init() };
        let mut dec_handle = unsafe { MaybeUninit::zeroed().assume_init() };
        
        let enc_cipher = unsafe { cmox_ecb_construct(&mut enc_handle as *mut _, CMOX_SM4_ECB_ENC) };
        let dec_cipher = unsafe { cmox_ecb_construct(&mut dec_handle as *mut _, CMOX_SM4_ECB_DEC) };

        if enc_cipher.is_null() || dec_cipher.is_null() {
            return Err(crate::error::CipherError::Internal.into());
        }

        unsafe {
            CipherResult::from_rv(cmox_cipher_init(enc_cipher))?;
            CipherResult::from_rv(cmox_cipher_init(dec_cipher))?;
            CipherResult::from_rv(cmox_cipher_setKey(enc_cipher, key.as_ptr(), key.len()))?;
            CipherResult::from_rv(cmox_cipher_setKey(dec_cipher, key.as_ptr(), key.len()))?;
        }

        Ok(Self {
            enc_handle,
            dec_handle,
            enc_cipher,
            dec_cipher,
        })
    }

    /// Encrypt a single block in-place
    pub fn encrypt_block_inplace(&self, block: &mut Block<Self>) -> crate::Result<()> {
        let mut output_len = block.len();
        unsafe {
            Ok(CipherResult::from_rv(cmox_cipher_append(
                self.enc_cipher,
                block.as_ptr(),
                block.len(),
                block.as_mut_ptr(),
                &mut output_len,
            ))?)
        }
    }

    /// Decrypt a single block in-place
    pub fn decrypt_block_inplace(&self, block: &mut Block<Self>) -> crate::Result<()> {
        let mut output_len = block.len();
        unsafe {
            Ok(CipherResult::from_rv(cmox_cipher_append(
                self.dec_cipher,
                block.as_ptr(),
                block.len(),
                block.as_mut_ptr(),
                &mut output_len,
            ))?)
        }
    }

    /// Encrypt a single block
    pub fn encrypt_block(&self, input: &Block<Self>) -> crate::Result<Block<Self>> {
        let mut output = *input;
        self.encrypt_block_inplace(&mut output)?;
        Ok(output)
    }

    /// Decrypt a single block
    pub fn decrypt_block(&self, input: &Block<Self>) -> crate::Result<Block<Self>> {
        let mut output = *input;
        self.decrypt_block_inplace(&mut output)?;
        Ok(output)
    }
}

impl Drop for Sm4 {
    fn drop(&mut self) {
        unsafe {
            cmox_cipher_cleanup(self.enc_cipher);
            cmox_cipher_cleanup(self.dec_cipher);
        }
    }
}

impl Clone for Sm4 {
    fn clone(&self) -> Self {
        // For block ciphers, we need to extract the key somehow
        // This is a simplified implementation that requires reinitialization
        panic!("Clone not supported for initialized cipher - create new instance");
    }
}

impl fmt::Debug for Sm4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sm4")
            .finish()
    }
}
