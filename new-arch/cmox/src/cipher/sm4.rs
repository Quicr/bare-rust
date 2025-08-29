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
    initialized: bool,
}

impl KeySizeUser for Sm4 {
    type KeySize = U16;
}

impl BlockSizeUser for Sm4 {
    type BlockSize = U16;
}

impl KeyInit for Sm4 {
    fn new(key: &Key<Self>) -> Self {
        let mut cipher = Self {
            enc_handle: unsafe { MaybeUninit::zeroed().assume_init() },
            dec_handle: unsafe { MaybeUninit::zeroed().assume_init() },
            enc_cipher: core::ptr::null_mut(),
            dec_cipher: core::ptr::null_mut(),
            initialized: false,
        };

        cipher
            .init_with_key(key)
            .expect("Failed to initialize SM4 cipher");
        cipher
    }
}

impl Sm4 {
    /// Create a new SM4 cipher with the given key
    pub fn new_with_key(key: &[u8]) -> crate::Result<Self> {
        let mut cipher = Self {
            enc_handle: unsafe { MaybeUninit::zeroed().assume_init() },
            dec_handle: unsafe { MaybeUninit::zeroed().assume_init() },
            enc_cipher: core::ptr::null_mut(),
            dec_cipher: core::ptr::null_mut(),
            initialized: false,
        };

        cipher.init_with_key(key)?;
        Ok(cipher)
    }

    fn init_with_key(&mut self, key: &[u8]) -> crate::Result<()> {
        ensure_initialized()?;

        // Initialize encryption handle
        self.enc_cipher =
            unsafe { cmox_ecb_construct(&mut self.enc_handle as *mut _, CMOX_SM4_ECB_ENC) };

        if self.enc_cipher.is_null() {
            return Err(crate::error::CipherError::Internal.into());
        }

        // Initialize decryption handle
        self.dec_cipher =
            unsafe { cmox_ecb_construct(&mut self.dec_handle as *mut _, CMOX_SM4_ECB_DEC) };

        if self.dec_cipher.is_null() {
            return Err(crate::error::CipherError::Internal.into());
        }

        // Initialize encryption
        let result = unsafe { cmox_cipher_init(self.enc_cipher) };
        CipherResult::from_rv(result)?;

        // Initialize decryption
        let result = unsafe { cmox_cipher_init(self.dec_cipher) };
        CipherResult::from_rv(result)?;

        // Set encryption key
        let result = unsafe { cmox_cipher_setKey(self.enc_cipher, key.as_ptr(), key.len()) };
        CipherResult::from_rv(result)?;

        // Set decryption key
        let result = unsafe { cmox_cipher_setKey(self.dec_cipher, key.as_ptr(), key.len()) };
        CipherResult::from_rv(result)?;

        self.initialized = true;
        Ok(())
    }

    /// Encrypt a single block in-place
    pub fn encrypt_block_inplace(&self, block: &mut Block<Self>) -> crate::Result<()> {
        if !self.initialized {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut output_len = block.len();
        let result = unsafe {
            cmox_cipher_append(
                self.enc_cipher,
                block.as_ptr(),
                block.len(),
                block.as_mut_ptr(),
                &mut output_len,
            )
        };

        Ok(CipherResult::from_rv(result)?)
    }

    /// Decrypt a single block in-place
    pub fn decrypt_block_inplace(&self, block: &mut Block<Self>) -> crate::Result<()> {
        if !self.initialized {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut output_len = block.len();
        let result = unsafe {
            cmox_cipher_append(
                self.dec_cipher,
                block.as_ptr(),
                block.len(),
                block.as_mut_ptr(),
                &mut output_len,
            )
        };

        Ok(CipherResult::from_rv(result)?)
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
        if self.initialized {
            unsafe {
                cmox_cipher_cleanup(self.enc_cipher);
                cmox_cipher_cleanup(self.dec_cipher);
            }
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
            .field("initialized", &self.initialized)
            .finish()
    }
}
