//! CMOX-based hash functions
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
use crate::error::{FromRetval, HashResult};
use cmox_sys::*;
use core::mem::MaybeUninit;
use digest::{
    consts::{U20, U28, U32, U48, U64},
    generic_array::ArrayLength,
    FixedOutput, HashMarker, Output, OutputSizeUser, Update,
};

pub trait HashType {
    type RawHandle;
    type Size: ArrayLength<u8>;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t;
}

macro_rules! hash {
    ($hash:ident, $type:ident, $handle:ty, $size:ty, $construct:ident) => {
        pub type $hash = Hash<$type>;

        pub struct $type;

        impl HashType for $type {
            type RawHandle = $handle;
            type Size = $size;
            fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
                unsafe { $construct(raw_handle as *mut _) }
            }
        }
    };
}

hash! { Sha1, Sha1Type, cmox_sha1_handle_t, U20, cmox_sha1_construct }
hash! { Sha224, Sha224Type, cmox_sha224_handle_t, U28, cmox_sha224_construct }
hash! { Sha256, Sha256Type, cmox_sha256_handle_t, U32, cmox_sha256_construct }
hash! { Sha384, Sha384Type, cmox_sha384_handle_t, U48, cmox_sha384_construct }
hash! { Sha512, Sha512Type, cmox_sha512_handle_t, U64, cmox_sha512_construct }
hash! { Sha512_224, Sha512_224Type, cmox_sha512_handle_t, U28, cmox_sha512_224_construct }
hash! { Sha512_256, Sha512_256Type, cmox_sha512_handle_t, U32, cmox_sha512_256_construct }
hash! { Sha3_224, Sha3_224Type, cmox_sha3_handle_t, U28, cmox_sha3_224_construct }
hash! { Sha3_256, Sha3_256Type, cmox_sha3_handle_t, U32, cmox_sha3_256_construct }
hash! { Sha3_384, Sha3_384Type, cmox_sha3_handle_t, U48, cmox_sha3_384_construct }
hash! { Sha3_512, Sha3_512Type, cmox_sha3_handle_t, U64, cmox_sha3_512_construct }
hash! { Sm3, Sm3Type, cmox_sm3_handle_t, U32, cmox_sm3_construct }

pub struct Hash<H: HashType> {
    raw_handle: <H as HashType>::RawHandle,
    hash_handle: *mut cmox_hash_handle_t,
}

impl<H: HashType> Default for Hash<H> {
    fn default() -> Self {
        ensure_initialized().expect("CMOX library not initialized");

        let mut raw_handle: H::RawHandle = unsafe { MaybeUninit::zeroed().assume_init() };
        let hash_handle = H::construct(&mut raw_handle);

        if hash_handle.is_null() {
            panic!("Failed to construct hash handle");
        }

        unsafe { HashResult::from_rv(cmox_hash_init(hash_handle)).expect("Hash init failed") }

        Self {
            raw_handle,
            hash_handle,
        }
    }
}

impl<H: HashType> Hash<H> {
    fn append(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        unsafe {
            HashResult::from_rv(cmox_hash_append(
                self.hash_handle,
                data.as_ptr(),
                data.len(),
            ))
            .expect("Hash update failed");
        }
    }

    fn generate(self, out: &mut Output<Self>) {
        let mut digest_len = out.len();
        unsafe {
            HashResult::from_rv(cmox_hash_generateTag(
                self.hash_handle,
                out.as_mut_ptr(),
                &mut digest_len as *mut usize,
            ))
            .expect("Hash finalization failed");
        }
    }

    fn cleanup(&mut self) {
        unsafe { cmox_hash_cleanup(self.hash_handle) };
    }
}

impl<H: HashType> HashMarker for Hash<H> {}

impl<H: HashType> OutputSizeUser for Hash<H> {
    type OutputSize = H::Size;
}

impl<H: HashType> Update for Hash<H> {
    fn update(&mut self, data: &[u8]) {
        self.append(data);
    }
}

impl<H: HashType> FixedOutput for Hash<H> {
    fn finalize_into(self, out: &mut Output<Self>) {
        self.generate(out);
    }
}

impl<H: HashType> Drop for Hash<H> {
    fn drop(&mut self) {
        self.cleanup();
    }
}
