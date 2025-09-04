#![allow(missing_docs)]

use crate::ensure_initialized;
use crate::error::{EccResult, FromRetval, Result};
use crate::hash::{Sha256, Sha384, Sha512};
use cmox_sys::*;
use core::mem::MaybeUninit;
use digest::Digest;
use elliptic_curve::{
    consts::{U133, U32, U48, U64, U65, U97},
    generic_array::{ArrayLength, GenericArray},
};
use heapless::Vec;
use rand_core::CryptoRngCore;
use signature::{RandomizedSigner, Verifier};

pub trait VecLike: Default {
    fn as_ptr(&self) -> *const u8;
    fn as_mut_ptr(&mut self) -> *mut u8;
    fn len(&self) -> usize;
}

impl<const N: usize> VecLike for Vec<u8, N> {
    fn as_ptr(&self) -> *const u8 {
        Vec::as_ptr(self)
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        Vec::as_mut_ptr(self)
    }

    fn len(&self) -> usize {
        // XXX(RLB) For some reason, just calling Vec::len causes infinite recursion.
        Vec::as_slice(self).len()
    }
}

pub trait Curve: Default {
    fn cmox_impl() -> cmox_ecc_impl_t;
    fn cmox_math() -> cmox_math_funcs_t;
    type Signature: VecLike;
    type PrivateKeyLength: ArrayLength<u8>;
    type PublicKeyLength: ArrayLength<u8>;
    type Hash: Digest;
}

#[derive(Default)]
pub struct P256;

impl Curve for P256 {
    fn cmox_impl() -> cmox_ecc_impl_t {
        unsafe { CMOX_ECC_SECP256R1_LOWMEM }
    }
    fn cmox_math() -> cmox_math_funcs_t {
        unsafe { CMOX_MATH_FUNCS_FAST }
    }
    type Signature = Vec<u8, 72>;
    type PrivateKeyLength = U32;
    type PublicKeyLength = U65;
    type Hash = Sha256;
}

#[derive(Default)]
pub struct P384;

impl Curve for P384 {
    fn cmox_impl() -> cmox_ecc_impl_t {
        unsafe { CMOX_ECC_SECP384R1_LOWMEM }
    }
    fn cmox_math() -> cmox_math_funcs_t {
        unsafe { CMOX_MATH_FUNCS_FAST }
    }
    type Signature = Vec<u8, 104>;
    type PrivateKeyLength = U48;
    type PublicKeyLength = U97;
    type Hash = Sha384;
}

#[derive(Default)]
pub struct P521;

impl Curve for P521 {
    fn cmox_impl() -> cmox_ecc_impl_t {
        unsafe { CMOX_ECC_SECP521R1_LOWMEM }
    }
    fn cmox_math() -> cmox_math_funcs_t {
        unsafe { CMOX_MATH_FUNCS_FAST }
    }
    type Signature = Vec<u8, 139>;
    type PrivateKeyLength = U64;
    type PublicKeyLength = U133;
    type Hash = Sha512;
}

pub type Seed<C> = GenericArray<u8, <C as Curve>::PrivateKeyLength>;
pub type PrivateKeyData<C> = GenericArray<u8, <C as Curve>::PrivateKeyLength>;
pub type PublicKeyData<C> = GenericArray<u8, <C as Curve>::PublicKeyLength>;
pub type Signature<C> = <C as Curve>::Signature;

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
        let mut public_key_len = public_key.0.len();

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
                public_key.0.as_mut_ptr(),
                &mut public_key_len,
            )
        };

        EccResult::from_rv(rv)?;

        Ok((private_key, public_key))
    }
}

impl<C: Curve> RandomizedSigner<Signature<C>> for PrivateKey<C> {
    fn try_sign_with_rng(
        &self,
        rng: &mut impl CryptoRngCore,
        message: &[u8],
    ) -> core::result::Result<Signature<C>, signature::Error> {
        ensure_initialized().map_err(|_| signature::Error::new())?;

        let digest = C::Hash::digest(message);

        let mut signature: Signature<C> = Default::default();
        let mut signature_len = signature.len();

        let mut random_k: Seed<C> = Default::default();
        rng.fill_bytes(&mut random_k);

        let rv = unsafe {
            let mut working_buffer = [0u8; 2048];
            let mut ctx = EccContext::new::<C>(&mut working_buffer);
            cmox_ecdsa_sign(
                ctx.context(),
                C::cmox_impl(),
                random_k.as_ptr(),
                random_k.len(),
                self.0.as_ptr(),
                self.0.len(),
                digest.as_ptr(),
                digest.len(),
                signature.as_mut_ptr(),
                &mut signature_len,
            )
        };

        EccResult::from_rv(rv).map_err(|_| signature::Error::new())?;

        Ok(signature)
    }
}

#[derive(Default)]
pub struct PublicKey<C: Curve>(pub PublicKeyData<C>);

impl<C: Curve> Verifier<Signature<C>> for PublicKey<C> {
    fn verify(
        &self,
        message: &[u8],
        signature: &Signature<C>,
    ) -> core::result::Result<(), signature::Error> {
        ensure_initialized().map_err(|_| signature::Error::new())?;

        let digest = C::Hash::digest(message);

        let mut fault_check: u32 = 0xffffffff;
        let rv = unsafe {
            let mut working_buffer = [0u8; 2048];
            let mut ctx = EccContext::new::<C>(&mut working_buffer);
            cmox_ecdsa_verify(
                ctx.context(),
                C::cmox_impl(),
                self.0.as_ptr(),
                self.0.len(),
                digest.as_ptr(),
                digest.len(),
                signature.as_ptr(),
                signature.len(),
                &mut fault_check,
            )
        };

        EccResult::from_rv(rv).map_err(|_| signature::Error::new())?;

        if rv != fault_check {
            return Err(signature::Error::new());
        }

        Ok(())
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
