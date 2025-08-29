//! AEAD (Authenticated Encryption with Associated Data) implementations
//!
//! This module provides idiomatic Rust wrappers for CMOX AEAD ciphers,
//! implementing the standard `aead` crate traits.

pub mod gcm;
pub use gcm::{Aes128Gcm, Aes192Gcm, Aes256Gcm};

pub mod ccm;
pub use ccm::{Aes128Ccm, Aes192Ccm, Aes256Ccm};

pub mod chacha20poly1305;
pub use chacha20poly1305::ChaCha20Poly1305;
