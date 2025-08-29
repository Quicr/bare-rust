//! AES block cipher implementations

use crate::error::{CipherError, CipherResult, FromRetval};
use crate::{ensure_initialized, Result};
use cipher::{
    consts::{U16, U24, U32},
    Block, BlockSizeUser, Key, KeyInit, KeySizeUser,
};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;

/// AES-128 cipher
pub struct Aes128 {
    enc_handle: cmox_ecb_handle_t,
    dec_handle: cmox_ecb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-192 cipher
pub struct Aes192 {
    enc_handle: cmox_ecb_handle_t,
    dec_handle: cmox_ecb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-256 cipher
pub struct Aes256 {
    enc_handle: cmox_ecb_handle_t,
    dec_handle: cmox_ecb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

// Helper macro to implement AES variants
macro_rules! impl_aes {
    ($name:ident, $key_size:ty, $key_len:expr, $enc_impl:ident, $dec_impl:ident) => {
        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl BlockSizeUser for $name {
            type BlockSize = U16;
        }

        impl KeyInit for $name {
            fn new(key: &Key<Self>) -> Self {
                ensure_initialized().expect("CMOX library not initialized");

                let mut enc_handle = unsafe { MaybeUninit::zeroed().assume_init() };
                let mut dec_handle = unsafe { MaybeUninit::zeroed().assume_init() };

                // Initialize encryption handle
                let enc_cipher = unsafe { cmox_ecb_construct(&mut enc_handle as *mut _, $enc_impl) };
                if enc_cipher.is_null() {
                    panic!("Failed to construct encryption handle");
                }

                // Initialize decryption handle
                let dec_cipher = unsafe { cmox_ecb_construct(&mut dec_handle as *mut _, $dec_impl) };
                if dec_cipher.is_null() {
                    panic!("Failed to construct decryption handle");
                }

                unsafe {
                    // Initialize encryption and decryption
                    CipherResult::from_rv(cmox_cipher_init(enc_cipher))
                        .expect("Failed to initialize encryption cipher");
                    CipherResult::from_rv(cmox_cipher_init(dec_cipher))
                        .expect("Failed to initialize decryption cipher");

                    // Set encryption and decryption keys
                    CipherResult::from_rv(cmox_cipher_setKey(enc_cipher, key.as_ptr(), key.len()))
                        .expect("Failed to set encryption key");
                    CipherResult::from_rv(cmox_cipher_setKey(dec_cipher, key.as_ptr(), key.len()))
                        .expect("Failed to set decryption key");
                }

                Self {
                    enc_handle,
                    dec_handle,
                    enc_cipher,
                    dec_cipher,
                }
            }
        }

        impl $name {
            /// Create a new AES cipher with the given key
            pub fn new_with_key(key: &[u8]) -> Result<Self> {
                ensure_initialized()?;

                let mut enc_handle = unsafe { MaybeUninit::zeroed().assume_init() };
                let mut dec_handle = unsafe { MaybeUninit::zeroed().assume_init() };

                // Initialize encryption handle
                let enc_cipher = unsafe { cmox_ecb_construct(&mut enc_handle as *mut _, $enc_impl) };
                if enc_cipher.is_null() {
                    return Err(CipherError::Internal.into());
                }

                // Initialize decryption handle
                let dec_cipher = unsafe { cmox_ecb_construct(&mut dec_handle as *mut _, $dec_impl) };
                if dec_cipher.is_null() {
                    return Err(CipherError::Internal.into());
                }

                unsafe {
                    // Initialize encryption and decryption
                    CipherResult::from_rv(cmox_cipher_init(enc_cipher))?;
                    CipherResult::from_rv(cmox_cipher_init(dec_cipher))?;

                    // Set encryption and decryption keys
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
            pub fn encrypt_block_inplace(&self, block: &mut Block<Self>) -> Result<()> {
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

                CipherResult::from_rv(result).map_err(Into::into)
            }

            /// Decrypt a single block in-place
            pub fn decrypt_block_inplace(&self, block: &mut Block<Self>) -> Result<()> {
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

                CipherResult::from_rv(result).map_err(Into::into)
            }

            /// Encrypt a single block
            pub fn encrypt_block(&self, input: &Block<Self>) -> Result<Block<Self>> {
                let mut output = *input;
                self.encrypt_block_inplace(&mut output)?;
                Ok(output)
            }

            /// Decrypt a single block
            pub fn decrypt_block(&self, input: &Block<Self>) -> Result<Block<Self>> {
                let mut output = *input;
                self.decrypt_block_inplace(&mut output)?;
                Ok(output)
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    cmox_cipher_cleanup(self.enc_cipher);
                    cmox_cipher_cleanup(self.dec_cipher);
                }
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                // For block ciphers, we need to extract the key somehow
                // This is a simplified implementation that requires reinitialization
                panic!("Clone not supported for initialized cipher - create new instance");
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .finish()
            }
        }
    };
}

// Implement all AES variants for ECB mode
impl_aes!(Aes128, U16, 16, CMOX_AESFAST_ECB_ENC, CMOX_AESFAST_ECB_DEC);
impl_aes!(Aes192, U24, 24, CMOX_AESFAST_ECB_ENC, CMOX_AESFAST_ECB_DEC);
impl_aes!(Aes256, U32, 32, CMOX_AESFAST_ECB_ENC, CMOX_AESFAST_ECB_DEC);

// Additional cipher modes

/// AES-128 CBC mode
pub struct Aes128Cbc {
    enc_handle: cmox_cbc_handle_t,
    dec_handle: cmox_cbc_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-192 CBC mode
pub struct Aes192Cbc {
    enc_handle: cmox_cbc_handle_t,
    dec_handle: cmox_cbc_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-256 CBC mode
pub struct Aes256Cbc {
    enc_handle: cmox_cbc_handle_t,
    dec_handle: cmox_cbc_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-128 CTR mode
pub struct Aes128Ctr {
    handle: cmox_ctr_handle_t,
    cipher: *mut cmox_cipher_handle_t,
}

/// AES-192 CTR mode
pub struct Aes192Ctr {
    handle: cmox_ctr_handle_t,
    cipher: *mut cmox_cipher_handle_t,
}

/// AES-256 CTR mode
pub struct Aes256Ctr {
    handle: cmox_ctr_handle_t,
    cipher: *mut cmox_cipher_handle_t,
}

/// AES-128 CFB mode
pub struct Aes128Cfb {
    enc_handle: cmox_cfb_handle_t,
    dec_handle: cmox_cfb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-192 CFB mode
pub struct Aes192Cfb {
    enc_handle: cmox_cfb_handle_t,
    dec_handle: cmox_cfb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-256 CFB mode
pub struct Aes256Cfb {
    enc_handle: cmox_cfb_handle_t,
    dec_handle: cmox_cfb_handle_t,
    enc_cipher: *mut cmox_cipher_handle_t,
    dec_cipher: *mut cmox_cipher_handle_t,
}

/// AES-128 OFB mode
pub struct Aes128Ofb {
    handle: cmox_ofb_handle_t,
    cipher: *mut cmox_cipher_handle_t,
}

/// AES-192 OFB mode
pub struct Aes192Ofb {
    handle: cmox_ofb_handle_t,
    cipher: *mut cmox_cipher_handle_t,
}

/// AES-256 OFB mode
pub struct Aes256Ofb {
    handle: cmox_ofb_handle_t,
    cipher: *mut cmox_cipher_handle_t,
}

// Helper macro for modes that need IV and support both encryption and decryption
macro_rules! impl_aes_iv_mode {
    ($name:ident, $key_size:ty, $key_len:expr, $handle_type:ty, $construct_fn:ident, $enc_impl:ident, $dec_impl:ident) => {
        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl BlockSizeUser for $name {
            type BlockSize = U16;
        }

        impl $name {
            /// Create a new cipher with the given key
            pub fn new_with_key(key: &[u8]) -> Result<Self> {
                ensure_initialized()?;

                let mut enc_handle = unsafe { MaybeUninit::zeroed().assume_init() };
                let mut dec_handle = unsafe { MaybeUninit::zeroed().assume_init() };

                // Initialize encryption handle
                let enc_cipher = unsafe { $construct_fn(&mut enc_handle as *mut _, $enc_impl) };
                if enc_cipher.is_null() {
                    return Err(CipherError::Internal.into());
                }

                // Initialize decryption handle
                let dec_cipher = unsafe { $construct_fn(&mut dec_handle as *mut _, $dec_impl) };
                if dec_cipher.is_null() {
                    return Err(CipherError::Internal.into());
                }

                unsafe {
                    // Initialize encryption and decryption
                    CipherResult::from_rv(cmox_cipher_init(enc_cipher))?;
                    CipherResult::from_rv(cmox_cipher_init(dec_cipher))?;

                    // Set encryption and decryption keys
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


            /// Encrypt data with the given IV
            pub fn encrypt(
                &self,
                iv: &[u8; 16],
                plaintext: &[u8],
                output: &mut [u8],
            ) -> Result<usize> {
                if output.len() < plaintext.len() {
                    return Err(CipherError::BadInputSize.into());
                }

                let mut output_len = plaintext.len();

                unsafe {
                    // Set IV and encrypt
                    CipherResult::from_rv(cmox_cipher_setIV(self.enc_cipher, iv.as_ptr(), iv.len()))?;
                    CipherResult::from_rv(cmox_cipher_append(
                        self.enc_cipher,
                        plaintext.as_ptr(),
                        plaintext.len(),
                        output.as_mut_ptr(),
                        &mut output_len,
                    ))?;
                }

                Ok(output_len)
            }

            /// Decrypt data with the given IV
            pub fn decrypt(
                &self,
                iv: &[u8; 16],
                ciphertext: &[u8],
                output: &mut [u8],
            ) -> Result<usize> {
                if output.len() < ciphertext.len() {
                    return Err(CipherError::BadInputSize.into());
                }

                let mut output_len = ciphertext.len();

                unsafe {
                    // Set IV and decrypt
                    CipherResult::from_rv(cmox_cipher_setIV(self.dec_cipher, iv.as_ptr(), iv.len()))?;
                    CipherResult::from_rv(cmox_cipher_append(
                        self.dec_cipher,
                        ciphertext.as_ptr(),
                        ciphertext.len(),
                        output.as_mut_ptr(),
                        &mut output_len,
                    ))?;
                }

                Ok(output_len)
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    cmox_cipher_cleanup(self.enc_cipher);
                    cmox_cipher_cleanup(self.dec_cipher);
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .finish()
            }
        }
    };
}

// Helper macro for stream cipher modes (CTR, OFB) that work the same for encryption/decryption
macro_rules! impl_aes_stream_mode {
    ($name:ident, $key_size:ty, $key_len:expr, $handle_type:ty, $construct_fn:ident, $impl_const:ident) => {
        impl KeySizeUser for $name {
            type KeySize = $key_size;
        }

        impl BlockSizeUser for $name {
            type BlockSize = U16;
        }

        impl $name {
            /// Create a new cipher with the given key
            pub fn new_with_key(key: &[u8]) -> Result<Self> {
                ensure_initialized()?;

                let mut handle = unsafe { MaybeUninit::zeroed().assume_init() };

                // Initialize cipher handle
                let cipher = unsafe { $construct_fn(&mut handle as *mut _, $impl_const) };
                if cipher.is_null() {
                    return Err(CipherError::Internal.into());
                }

                unsafe {
                    // Initialize cipher and set key
                    CipherResult::from_rv(cmox_cipher_init(cipher))?;
                    CipherResult::from_rv(cmox_cipher_setKey(cipher, key.as_ptr(), key.len()))?;
                }

                Ok(Self {
                    handle,
                    cipher,
                })
            }


            /// Encrypt or decrypt data with the given IV/nonce (same operation for stream ciphers)
            pub fn process(&self, iv: &[u8; 16], data: &[u8], output: &mut [u8]) -> Result<usize> {
                if output.len() < data.len() {
                    return Err(CipherError::BadInputSize.into());
                }

                let mut output_len = data.len();

                unsafe {
                    // Set IV/nonce and process data
                    CipherResult::from_rv(cmox_cipher_setIV(self.cipher, iv.as_ptr(), iv.len()))?;
                    CipherResult::from_rv(cmox_cipher_append(
                        self.cipher,
                        data.as_ptr(),
                        data.len(),
                        output.as_mut_ptr(),
                        &mut output_len,
                    ))?;
                }

                Ok(output_len)
            }

            /// Encrypt data (alias for process)
            pub fn encrypt(
                &self,
                iv: &[u8; 16],
                plaintext: &[u8],
                output: &mut [u8],
            ) -> Result<usize> {
                self.process(iv, plaintext, output)
            }

            /// Decrypt data (alias for process)
            pub fn decrypt(
                &self,
                iv: &[u8; 16],
                ciphertext: &[u8],
                output: &mut [u8],
            ) -> Result<usize> {
                self.process(iv, ciphertext, output)
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    cmox_cipher_cleanup(self.cipher);
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .finish()
            }
        }
    };
}

// Implement CBC mode variants
impl_aes_iv_mode!(
    Aes128Cbc,
    U16,
    16,
    cmox_cbc_handle_t,
    cmox_cbc_construct,
    CMOX_AESFAST_CBC_ENC,
    CMOX_AESFAST_CBC_DEC
);
impl_aes_iv_mode!(
    Aes192Cbc,
    U24,
    24,
    cmox_cbc_handle_t,
    cmox_cbc_construct,
    CMOX_AESFAST_CBC_ENC,
    CMOX_AESFAST_CBC_DEC
);
impl_aes_iv_mode!(
    Aes256Cbc,
    U32,
    32,
    cmox_cbc_handle_t,
    cmox_cbc_construct,
    CMOX_AESFAST_CBC_ENC,
    CMOX_AESFAST_CBC_DEC
);

// Implement CFB mode variants
impl_aes_iv_mode!(
    Aes128Cfb,
    U16,
    16,
    cmox_cfb_handle_t,
    cmox_cfb_construct,
    CMOX_AESFAST_CFB_ENC,
    CMOX_AESFAST_CFB_DEC
);
impl_aes_iv_mode!(
    Aes192Cfb,
    U24,
    24,
    cmox_cfb_handle_t,
    cmox_cfb_construct,
    CMOX_AESFAST_CFB_ENC,
    CMOX_AESFAST_CFB_DEC
);
impl_aes_iv_mode!(
    Aes256Cfb,
    U32,
    32,
    cmox_cfb_handle_t,
    cmox_cfb_construct,
    CMOX_AESFAST_CFB_ENC,
    CMOX_AESFAST_CFB_DEC
);

// Implement CTR mode variants (stream cipher)
impl_aes_stream_mode!(
    Aes128Ctr,
    U16,
    16,
    cmox_ctr_handle_t,
    cmox_ctr_construct,
    CMOX_AESFAST_CTR_ENC
);
impl_aes_stream_mode!(
    Aes192Ctr,
    U24,
    24,
    cmox_ctr_handle_t,
    cmox_ctr_construct,
    CMOX_AESFAST_CTR_ENC
);
impl_aes_stream_mode!(
    Aes256Ctr,
    U32,
    32,
    cmox_ctr_handle_t,
    cmox_ctr_construct,
    CMOX_AESFAST_CTR_ENC
);

// Implement OFB mode variants (stream cipher)
impl_aes_stream_mode!(
    Aes128Ofb,
    U16,
    16,
    cmox_ofb_handle_t,
    cmox_ofb_construct,
    CMOX_AESFAST_OFB_ENC
);
impl_aes_stream_mode!(
    Aes192Ofb,
    U24,
    24,
    cmox_ofb_handle_t,
    cmox_ofb_construct,
    CMOX_AESFAST_OFB_ENC
);
impl_aes_stream_mode!(
    Aes256Ofb,
    U32,
    32,
    cmox_ofb_handle_t,
    cmox_ofb_construct,
    CMOX_AESFAST_OFB_ENC
);
