#![allow(missing_docs)]

use crate::ensure_initialized;
use crate::error::{EccResult, FromRetval, Result};
use cmox_sys::*;
use core::mem::MaybeUninit;
use elliptic_curve::{
    consts::{U114, U32, U56, U64},
    generic_array::{ArrayLength, GenericArray},
};
use rand_core::CryptoRngCore;
use signature::{Signer, Verifier};

pub trait Curve: Default {
    fn cmox_impl() -> cmox_ecc_impl_t;
    fn cmox_math() -> cmox_math_funcs_t;
    type SignatureLength: ArrayLength<u8>;
    type PrivateKeyLength: ArrayLength<u8>;
    type PublicKeyLength: ArrayLength<u8>;
}

#[derive(Default)]
pub struct Ed25519;

impl Curve for Ed25519 {
    fn cmox_impl() -> cmox_ecc_impl_t {
        unsafe { CMOX_ECC_ED25519_OPT_LOWMEM }
    }
    fn cmox_math() -> cmox_math_funcs_t {
        unsafe { CMOX_MATH_FUNCS_FAST }
    }
    type SignatureLength = U64;
    type PrivateKeyLength = U32;
    type PublicKeyLength = U32;
}

#[derive(Default)]
pub struct Ed448;

impl Curve for Ed448 {
    fn cmox_impl() -> cmox_ecc_impl_t {
        unsafe { CMOX_ECC_ED448_LOWMEM }
    }
    fn cmox_math() -> cmox_math_funcs_t {
        unsafe { CMOX_MATH_FUNCS_FAST }
    }
    type SignatureLength = U114;
    type PrivateKeyLength = U56;
    type PublicKeyLength = U56;
}

pub type Seed<C> = GenericArray<u8, <C as Curve>::PrivateKeyLength>;
pub type PrivateKeyData<C> = GenericArray<u8, <C as Curve>::PrivateKeyLength>;
pub type PublicKeyData<C> = GenericArray<u8, <C as Curve>::PublicKeyLength>;
pub type Signature<C> = GenericArray<u8, <C as Curve>::SignatureLength>;

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
            cmox_eddsa_keyGen(
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

impl<C: Curve> Signer<Signature<C>> for PrivateKey<C> {
    fn try_sign(&self, message: &[u8]) -> core::result::Result<Signature<C>, signature::Error> {
        ensure_initialized().map_err(|_| signature::Error::new())?;

        let mut signature: Signature<C> = Default::default();
        let mut signature_len = signature.len();

        let rv = unsafe {
            let mut working_buffer = [0u8; 2048];
            let mut ctx = EccContext::new::<C>(&mut working_buffer);
            cmox_eddsa_sign(
                ctx.context(),
                C::cmox_impl(),
                self.0.as_ptr(),
                self.0.len(),
                message.as_ptr(),
                message.len(),
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

        let mut fault_check: u32 = 0xffffffff;
        let rv = unsafe {
            let mut working_buffer = [0u8; 2048];
            let mut ctx = EccContext::new::<C>(&mut working_buffer);
            cmox_eddsa_verify(
                ctx.context(),
                C::cmox_impl(),
                self.0.as_ptr(),
                self.0.len(),
                message.as_ptr(),
                message.len(),
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
