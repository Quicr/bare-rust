//! CMOX-based MAC functions
//!
//! This module provides access to the HMAC functions exposed by CMOX, with the following hash
//! functions:
//!
//! * SHA-1
//! * SHA-224
//! * SHA-256
//! * SHA-384
//! * SHA-512
//! * SHA-512_224
//! * SHA512_256
//!
//! KMAC is not implemented because its variable-length nature doesn't align well with the
//! fixed-length assumptions of the Rust Crypto MAC API.
//!
//! AES-CMAC is not implemented, becuase it was not clear from the CMOX documentation what inputs
//! it expects.
#![allow(missing_docs)]

use crate::ensure_initialized;
use crate::error::{FromRetval, MacResult};
use cipher::KeySizeUser;
use cmox_sys::*;
use core::mem::MaybeUninit;
use digest::{
    consts::{U20, U28, U32, U48, U64},
    generic_array::ArrayLength,
    FixedOutput, Key, KeyInit, MacMarker, Output, OutputSizeUser, Update,
};

pub trait MacType {
    type RawHandle;
    type KeySize: ArrayLength<u8>;
    type OutputSize: ArrayLength<u8>;
    fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_mac_handle_t;
}

macro_rules! mac {
    ($mac:ident, $type:ident, $handle:ident, $key_size:ident, $out_size:ident, $construct:ident, $param:ident) => {
        pub type $mac = MacImpl<$type>;

        pub struct $type;

        impl MacType for $type {
            type RawHandle = $handle;
            type KeySize = $key_size;
            type OutputSize = $out_size;
            fn construct(raw_handle: &mut Self::RawHandle) -> *mut cmox_mac_handle_t {
                unsafe { $construct(raw_handle as *mut _, $param) }
            }
        }
    };
}

macro_rules! hmac {
    ($mac:ident, $type:ident, $size:ident, $param:ident) => {
        mac! { $mac, $type, cmox_hmac_handle_t, $size, $size, cmox_hmac_construct, $param }
    };
}

hmac! { HmacSha1, HmacSha1Type, U20, CMOX_HMAC_SHA1 }
hmac! { HmacSha224, HmacSha224Type, U28, CMOX_HMAC_SHA224 }
hmac! { HmacSha256, HmacSha256Type, U32, CMOX_HMAC_SHA256 }
hmac! { HmacSha384, HmacSha384Type, U48, CMOX_HMAC_SHA384 }
hmac! { HmacSha512, HmacSha512Type, U64, CMOX_HMAC_SHA512 }
hmac! { HmacSha512_224, HmacSha512_224Type, U64, CMOX_HMAC_SHA512_224 }
hmac! { HmacSha512_256, HmacSha512_256Type, U64, CMOX_HMAC_SHA512_256 }
hmac! { HmacSm3, HmacSm3Type, U32, CMOX_HMAC_SM3 }

pub struct MacImpl<M: MacType> {
    raw_handle: M::RawHandle,
    mac_handle: *mut cmox_mac_handle_t,
}

impl<M: MacType> MacImpl<M> {
    fn new(key: &[u8]) -> Self {
        ensure_initialized().expect("CMOX library not initialized");

        let mut raw_handle: M::RawHandle = unsafe { MaybeUninit::zeroed().assume_init() };
        let mac_handle = M::construct(&mut raw_handle);

        if mac_handle.is_null() {
            panic!("Failed to construct MAC handle");
        }

        unsafe {
            MacResult::from_rv(cmox_mac_init(mac_handle)).expect("MAC init failed");
            MacResult::from_rv(cmox_mac_setKey(mac_handle, key.as_ptr(), key.len()))
                .expect("MAC set key failed");
        }

        Self {
            raw_handle,
            mac_handle,
        }
    }

    fn append(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        unsafe {
            MacResult::from_rv(cmox_mac_append(self.mac_handle, data.as_ptr(), data.len()))
                .expect("MAC update failed");
        }
    }

    fn generate(self, out: &mut Output<Self>) {
        let mut tag_len = out.len();
        unsafe {
            MacResult::from_rv(cmox_mac_generateTag(
                self.mac_handle,
                out.as_mut_ptr(),
                &mut tag_len,
            ))
            .expect("MAC generation failed")
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            cmox_mac_cleanup(self.mac_handle);
        }
    }
}

impl<M: MacType> MacMarker for MacImpl<M> {}

impl<M: MacType> KeySizeUser for MacImpl<M> {
    type KeySize = M::KeySize;
}

impl<M: MacType> OutputSizeUser for MacImpl<M> {
    type OutputSize = M::OutputSize;
}

impl<M: MacType> KeyInit for MacImpl<M> {
    fn new(key: &Key<Self>) -> Self {
        Self::new(key.as_slice())
    }

    fn new_from_slice(key: &[u8]) -> core::result::Result<Self, digest::InvalidLength> {
        Ok(Self::new(key))
    }
}

impl<M: MacType> Update for MacImpl<M> {
    fn update(&mut self, data: &[u8]) {
        self.append(data);
    }
}

impl<M: MacType> FixedOutput for MacImpl<M> {
    fn finalize_into(self, out: &mut Output<Self>) {
        self.generate(out);
    }
}

impl<M: MacType> Drop for MacImpl<M> {
    fn drop(&mut self) {
        self.cleanup();
    }
}
