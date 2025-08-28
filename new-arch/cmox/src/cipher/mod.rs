//! Block cipher implementations
//!
//! This module provides idiomatic Rust wrappers for CMOX block ciphers,
//! implementing the standard `cipher` crate traits.

pub mod aes;
pub use aes::{
    // ECB mode (basic block cipher interface)
    Aes128, Aes192, Aes256,
    // CBC mode (requires IV)
    Aes128Cbc, Aes192Cbc, Aes256Cbc,
    // CFB mode (requires IV) 
    Aes128Cfb, Aes192Cfb, Aes256Cfb,
    // CTR mode (stream cipher mode)
    Aes128Ctr, Aes192Ctr, Aes256Ctr,
    // OFB mode (stream cipher mode)
    Aes128Ofb, Aes192Ofb, Aes256Ofb,
};

pub mod sm4;
pub use sm4::Sm4;