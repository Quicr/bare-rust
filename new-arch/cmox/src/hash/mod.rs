//! Hash function implementations
//!
//! This module provides idiomatic Rust wrappers for CMOX hash functions,
//! implementing the standard `digest` crate traits.

pub mod sha1;
pub use sha1::{Sha1, Sha1Hash};

pub mod sha2;
pub use sha2::{
    Sha224, Sha224Hash, Sha256, Sha256Hash, Sha384, Sha384Hash, Sha512, Sha512Hash, Sha512_224,
    Sha512_224Hash, Sha512_256, Sha512_256Hash,
};

pub mod sha3;
pub use sha3::{
    Sha3_224, Sha3_224Hash, Sha3_256, Sha3_256Hash, Sha3_384, Sha3_384Hash, Sha3_512, Sha3_512Hash,
};

pub mod sm3;
pub use sm3::{Sm3, Sm3Hash};
