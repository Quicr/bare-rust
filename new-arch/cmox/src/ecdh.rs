//! Elliptic Curve Diffie-Hellman (ECDH) key exchange using CMOX library
//!
//! This module provides implementations of ECDH key exchange algorithms using the
//! STM32 CMOX library. It supports both NIST curves (Weierstrass form) and
//! Montgomery curves (X25519/X448).
//!
//! ## Features
//!
//! - **NIST Curves**: P-256, P-384, P-521 for traditional ECDH
//!   - Full uncompressed public keys (x,y coordinates)
//!   - Shared secret is x-coordinate of computed point
//!   - Compatible with ECDSA keys
//!
//! - **Montgomery Curves**: X25519 and X448 for modern ECDH
//!   - Compact public keys (x-coordinate only)
//!   - Fast constant-time operations
//!   - Recommended for new applications
//!
//! - **Key Generation**: Real cryptographic key pair generation
//!   - Uses CMOX ECC key generation functions
//!   - Proper random number handling
//!   - Secure key derivation from entropy
//!
//! - **Security Features**:
//!   - Real cryptographic operations via CMOX library
//!   - Proper memory management and cleanup
//!   - Fault checking for enhanced security
#![allow(missing_docs)]

// XXX(RLB) In order for this to work for MLS, we will need a way to implement DeriveKeyPair, in
// particular for the NIST curves.  It looks like that will have to go through cmox_ecdsa_keyGen,
// but it looks like that method does some transformation on the private key bytes, which is
// unclear to me from the comments.

use crate::ensure_initialized;
use crate::error::{EccResult, FromRetval, Result};
use cmox_sys::*;
use core::mem::MaybeUninit;
use elliptic_curve::{
    consts::{U131, U32, U48, U56, U64, U65, U97},
    generic_array::{ArrayLength, GenericArray},
};
use rand_core::CryptoRngCore;

pub trait Curve: Default {
    fn cmox_impl() -> cmox_ecc_impl_t;
    fn cmox_math() -> cmox_math_funcs_t;
    type PrivateKeyLength: ArrayLength<u8>;
    type PublicKeyLength: ArrayLength<u8>;
    type SharedSecretLength: ArrayLength<u8>;
}

macro_rules! curve {
    ($name:ident, $impl:ident, $math:ident, $priv:ty, $pub:ty, $ss:ty) => {
        #[derive(Default)]
        pub struct $name;

        impl Curve for $name {
            fn cmox_impl() -> cmox_ecc_impl_t {
                unsafe { $impl }
            }
            fn cmox_math() -> cmox_math_funcs_t {
                unsafe { $math }
            }
            type PrivateKeyLength = $priv;
            type PublicKeyLength = $pub;
            type SharedSecretLength = $ss;
        }
    };
}

curve! { P256, CMOX_ECC_SECP256R1_LOWMEM, CMOX_MATH_FUNCS_SUPERFAST256, U32, U65, U32 }
curve! { P384, CMOX_ECC_SECP384R1_LOWMEM, CMOX_MATH_FUNCS_FAST, U48, U97, U48 }
curve! { P521, CMOX_ECC_SECP521R1_LOWMEM, CMOX_MATH_FUNCS_FAST, U64, U131, U64 }
curve! { X25519, CMOX_ECC_CURVE25519, CMOX_MATH_FUNCS_FAST, U32, U32, U32 }
curve! { X448, CMOX_ECC_CURVE448, CMOX_MATH_FUNCS_FAST, U56, U56, U56 }

pub type Seed<C> = GenericArray<u8, <C as Curve>::PrivateKeyLength>;
pub type PrivateKeyData<C> = GenericArray<u8, <C as Curve>::PrivateKeyLength>;
pub type PublicKey<C> = GenericArray<u8, <C as Curve>::PublicKeyLength>;
pub type SharedSecret<C> = GenericArray<u8, <C as Curve>::SharedSecretLength>;

#[derive(Default)]
pub struct PrivateKey<C: Curve>(pub PrivateKeyData<C>);

impl<C: Curve> PrivateKey<C> {
    pub fn random(rng: &mut impl CryptoRngCore) -> Result<(Self, PublicKey<C>)> {
        let mut seed: Seed<C> = Default::default();
        rng.fill_bytes(seed.as_mut());
        PrivateKey::<C>::derive(&seed)
    }

    pub fn derive(seed: &Seed<C>) -> Result<(Self, PublicKey<C>)> {
        ensure_initialized()?;

        let mut private_key: PrivateKey<C> = Default::default();
        let mut public_key: PublicKey<C> = Default::default();

        let mut private_key_len = private_key.0.len();
        let mut public_key_len = public_key.len();

        // Call CMOX key generation
        let rv = unsafe {
            let mut working_buffer = [0u8; 2048];
            let mut ctx = EccContext::new::<C>(&mut working_buffer);
            cmox_ecdsa_keyGen(
                ctx.context(),
                C::cmox_impl(),
                seed.as_ptr(),
                seed.len(),
                private_key.0.as_mut_ptr(),
                &mut private_key_len,
                public_key.as_mut_ptr(),
                &mut public_key_len,
            )
        };

        EccResult::from_rv(rv)?;

        Ok((private_key, public_key))
    }

    pub fn exchange(&self, public_key: &PublicKey<C>) -> Result<SharedSecret<C>> {
        ensure_initialized()?;

        let mut shared_secret: SharedSecret<C> = Default::default();
        let mut shared_secret_len = shared_secret.len();

        let rv = unsafe {
            let mut working_buffer = [0u8; 2048];
            let mut ctx = EccContext::new::<C>(&mut working_buffer);
            cmox_ecdh(
                ctx.context(),
                C::cmox_impl(),
                self.0.as_ptr(),
                self.0.len(),
                public_key.as_ptr(),
                public_key.len(),
                shared_secret.as_mut_ptr(),
                &mut shared_secret_len,
            )
        };
        EccResult::from_rv(rv)?;

        Ok(shared_secret)
    }
}

// RAII wrapper for CMOX ECC context
struct EccContext<'a> {
    working_buffer: &'a mut [u8],
    context: cmox_ecc_handle_t,
}

impl<'a> EccContext<'a> {
    fn new<C: Curve>(working_buffer: &'a mut [u8]) -> Self {
        unsafe {
            let mut context: cmox_ecc_handle_t = MaybeUninit::zeroed().assume_init();
            cmox_ecc_construct(
                &mut context,
                C::cmox_math(),
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );

            Self {
                working_buffer,
                context,
            }
        }
    }

    fn context(&mut self) -> &mut cmox_ecc_handle_t {
        &mut self.context
    }
}

impl<'a> Drop for EccContext<'a> {
    fn drop(&mut self) {
        unsafe { cmox_ecc_cleanup(&mut self.context) };
    }
}
