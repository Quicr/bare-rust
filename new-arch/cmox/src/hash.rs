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
use core::fmt;
use core::mem::MaybeUninit;
use digest::{
    consts::{U20, U28, U32, U48, U64},
    generic_array::ArrayLength,
    FixedOutput, FixedOutputReset, HashMarker, Output, OutputSizeUser, Reset, Update,
};

pub type Sha1 = Hash<Sha1Type>;
pub type Sha224 = Hash<Sha224Type>;
pub type Sha256 = Hash<Sha256Type>;
pub type Sha3_224 = Hash<Sha3_224Type>;
pub type Sha3_256 = Hash<Sha3_256Type>;
pub type Sha3_384 = Hash<Sha3_384Type>;
pub type Sha3_512 = Hash<Sha3_512Type>;
pub type Sha384 = Hash<Sha384Type>;
pub type Sha512_224 = Hash<Sha512_224Type>;
pub type Sha512_256 = Hash<Sha512_256Type>;
pub type Sha512 = Hash<Sha512Type>;
pub type Sm3 = Hash<Sm3Type>;

pub trait HashType {
    type RawHandle;
    type Size: ArrayLength<u8>;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t;
}

pub struct Sha1Type;

impl HashType for Sha1Type {
    type RawHandle = cmox_sha1_handle_t;
    type Size = U20;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha1_construct(raw_handle as *mut _) }
    }
}

pub struct Sha224Type;

impl HashType for Sha224Type {
    type RawHandle = cmox_sha224_handle_t;
    type Size = U28;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha224_construct(raw_handle as *mut _) }
    }
}

pub struct Sha256Type;

impl HashType for Sha256Type {
    type RawHandle = cmox_sha256_handle_t;
    type Size = U32;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha256_construct(raw_handle as *mut _) }
    }
}

pub struct Sha3_224Type;

impl HashType for Sha3_224Type {
    type RawHandle = cmox_sha3_handle_t;
    type Size = U28;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha3_224_construct(raw_handle as *mut _) }
    }
}

pub struct Sha3_256Type;

impl HashType for Sha3_256Type {
    type RawHandle = cmox_sha3_handle_t;
    type Size = U32;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha3_256_construct(raw_handle as *mut _) }
    }
}

pub struct Sha3_384Type;

impl HashType for Sha3_384Type {
    type RawHandle = cmox_sha3_handle_t;
    type Size = U48;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha3_384_construct(raw_handle as *mut _) }
    }
}

pub struct Sha3_512Type;

impl HashType for Sha3_512Type {
    type RawHandle = cmox_sha3_handle_t;
    type Size = U64;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha3_512_construct(raw_handle as *mut _) }
    }
}

pub struct Sha384Type;

impl HashType for Sha384Type {
    type RawHandle = cmox_sha384_handle_t;
    type Size = U48;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha384_construct(raw_handle as *mut _) }
    }
}

pub struct Sha512Type;

impl HashType for Sha512Type {
    type RawHandle = cmox_sha512_handle_t;
    type Size = U64;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha512_construct(raw_handle as *mut _) }
    }
}

pub struct Sha512_224Type;

impl HashType for Sha512_224Type {
    type RawHandle = cmox_sha512_handle_t;
    type Size = U28;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha512_224_construct(raw_handle as *mut _) }
    }
}

pub struct Sha512_256Type;

impl HashType for Sha512_256Type {
    type RawHandle = cmox_sha512_handle_t;
    type Size = U32;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sha512_256_construct(raw_handle as *mut _) }
    }
}

pub struct Sm3Type;

impl HashType for Sm3Type {
    type RawHandle = cmox_sm3_handle_t;
    type Size = U32;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_hash_handle_t {
        unsafe { cmox_sm3_construct(raw_handle as *mut _) }
    }
}

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
            panic!("Failed to construct SHA-1 hash handle");
        }

        let mut h = Self {
            raw_handle,
            hash_handle,
        };

        h.init();
        h
    }
}

impl<H: HashType> Hash<H> {
    fn init(&mut self) {
        unsafe { HashResult::from_rv(cmox_hash_init(self.hash_handle)).expect("Hash reset failed") }
    }

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

    fn generate(&mut self, out: &mut Output<Self>) {
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

impl<H: HashType> Reset for Hash<H> {
    fn reset(&mut self) {
        self.cleanup();
        self.init();
    }
}

impl<H: HashType> Update for Hash<H> {
    fn update(&mut self, data: &[u8]) {
        self.append(data);
    }
}

impl<H: HashType> FixedOutput for Hash<H> {
    fn finalize_into(self, out: &mut Output<Self>) {
        let mut h = self;
        h.generate(out);
    }
}

impl<H: HashType> FixedOutputReset for Hash<H> {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        self.generate(out);
        Reset::reset(self);
    }
}

impl<H: HashType> Drop for Hash<H> {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl<H: HashType> fmt::Debug for Hash<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sha1").finish()
    }
}
