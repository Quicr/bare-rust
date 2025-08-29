//! Block cipher implementations
//!
//! This module provides idiomatic Rust wrappers for CMOX block ciphers,
//! implementing the standard `cipher` crate traits.

pub mod aes;
pub use aes::{
    // ECB mode (basic block cipher interface)
    Aes128,
    // CBC mode (requires IV)
    Aes128Cbc,
    // CFB mode (requires IV)
    Aes128Cfb,
    // CTR mode (stream cipher mode)
    Aes128Ctr,
    // OFB mode (stream cipher mode)
    Aes128Ofb,
    Aes192,
    Aes192Cbc,
    Aes192Cfb,
    Aes192Ctr,
    Aes192Ofb,
    Aes256,
    Aes256Cbc,
    Aes256Cfb,
    Aes256Ctr,
    Aes256Ofb,
};

pub mod sm4;
pub use sm4::Sm4;
