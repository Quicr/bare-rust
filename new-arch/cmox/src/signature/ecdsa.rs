use crate::ensure_initialized;
use crate::error::{EccError, EccResult, FromRetval};
use cmox_sys::*;
use core::mem::MaybeUninit;
use heapless::Vec;
use rand_core::{CryptoRng, RngCore};

/// Supported elliptic curves
#[derive(Copy, Clone, Debug)]
pub enum Curve {
    /// NIST P-256 curve (secp256r1) - ECDSA
    P256,
    /// NIST P-384 curve (secp384r1) - ECDSA
    P384,
    /// NIST P-521 curve (secp521r1) - ECDSA
    P521,
}

impl Curve {
    fn to_cmox_impl(self) -> cmox_ecc_impl_t {
        match self {
            Curve::P256 => unsafe { CMOX_ECC_SECP256R1_LOWMEM },
            Curve::P384 => unsafe { CMOX_ECC_SECP384R1_LOWMEM },
            Curve::P521 => unsafe { CMOX_ECC_SECP521R1_LOWMEM },
        }
    }

    fn math_funcs(self) -> cmox_math_funcs_t {
        match self {
            Curve::P256 => unsafe { CMOX_MATH_FUNCS_SUPERFAST256 },
            Curve::P384 => unsafe { CMOX_MATH_FUNCS_FAST },
            Curve::P521 => unsafe { CMOX_MATH_FUNCS_FAST },
        }
    }

    fn signature_len(self) -> usize {
        match self {
            Curve::P256 => 64,  // 2 * 32 bytes (r + s)
            Curve::P384 => 96,  // 2 * 48 bytes
            Curve::P521 => 132, // 2 * 66 bytes
        }
    }

    fn private_key_len(self) -> usize {
        match self {
            Curve::P256 => 32,
            Curve::P384 => 48,
            Curve::P521 => 66,
        }
    }

    fn public_key_len(self) -> usize {
        match self {
            Curve::P256 => 64,  // 2 * 32 bytes (x + y)
            Curve::P384 => 96,  // 2 * 48 bytes
            Curve::P521 => 132, // 2 * 66 bytes
        }
    }
}

/// ECDSA signature with fixed-size components (max P-521 size)
#[derive(Clone, Debug)]
pub struct EcdsaSignature(Vec<u8, 132>);

impl EcdsaSignature {
    /// Create signature from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        let mut signature = Vec::new();
        signature
            .extend_from_slice(bytes)
            .map_err(|_| EccError::BadParameter)?;

        Ok(Self(signature))
    }

    /// Get signature as bytes  
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// ECDSA signing operations
pub struct EcdsaSigningKey {
    curve: Curve,
    private_key: Vec<u8, 66>, // Max private key size for P-521
}

impl EcdsaSigningKey {
    /// Create a new ECDSA signing key
    pub fn new(curve: Curve, bytes: &[u8]) -> crate::Result<Self> {
        let mut private_key = Vec::new();
        private_key
            .extend_from_slice(bytes)
            .map_err(|_| EccError::BadParameter)?;

        Ok(Self { curve, private_key })
    }

    /// Sign a message digest using ECDSA
    ///
    /// # Arguments
    /// * `digest` - The message digest to sign
    /// * `rng` - Cryptographically secure random number generator for generating k value
    pub fn sign_digest<R>(&self, digest: &[u8], rng: &mut R) -> crate::Result<EcdsaSignature>
    where
        R: RngCore + CryptoRng,
    {
        ensure_initialized()?;

        // Create ECC context with working buffer
        // Buffer size needs to be sufficient for ECC operations - using 2KB for safety
        let mut working_buffer = [0u8; 2048];
        let mut ecc_ctx: cmox_ecc_handle_t = unsafe { MaybeUninit::zeroed().assume_init() };

        // Construct ECC context
        unsafe {
            cmox_ecc_construct(
                &mut ecc_ctx,
                self.curve.math_funcs(),
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );
        }

        let mut sig_len = self.curve.signature_len();
        let mut signature = Vec::new();
        let _ = signature.resize(sig_len, 0);

        // Generate cryptographically secure random bytes for k value
        let mut random_k = Vec::<u8, 64>::new();
        let _ = random_k.resize(self.private_key.len(), 0);
        rng.fill_bytes(&mut random_k);

        // Call CMOX ECDSA sign
        let result = unsafe {
            cmox_ecdsa_sign(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                random_k.as_ptr(),
                random_k.len(),
                self.private_key.as_ptr(),
                self.private_key.len(),
                digest.as_ptr(),
                digest.len(),
                signature.as_mut_ptr(),
                &mut sig_len,
            )
        };

        // Cleanup ECC context
        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        // Check result
        EccResult::from_rv(result)?;

        Ok(EcdsaSignature(signature))
    }
}

/// ECDSA verification operations
pub struct EcdsaVerifyingKey {
    curve: Curve,
    public_key: Vec<u8, 132>, // Max public key size for P-521
}

impl EcdsaVerifyingKey {
    /// Create a new ECDSA verifying key
    pub fn new(curve: Curve, bytes: &[u8]) -> crate::Result<Self> {
        let mut public_key = Vec::new();
        public_key
            .extend_from_slice(bytes)
            .map_err(|_| EccError::BadParameter)?;

        Ok(Self { curve, public_key })
    }

    /// Verify a signature against a message digest
    pub fn verify_digest(&self, digest: &[u8], signature: &EcdsaSignature) -> crate::Result<()> {
        ensure_initialized()?;

        // Check signature length
        if signature.as_bytes().len() != self.curve.signature_len() {
            return Err(crate::error::EccError::BadParameter.into());
        }

        // Create ECC context with working buffer
        let mut working_buffer = [0u8; 2048];
        let mut ecc_ctx: cmox_ecc_handle_t = unsafe { MaybeUninit::zeroed().assume_init() };

        // Construct ECC context
        unsafe {
            cmox_ecc_construct(
                &mut ecc_ctx,
                self.curve.math_funcs(),
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );
        }

        let mut fault_check: u32 = 0;

        // Call CMOX ECDSA verify
        let result = unsafe {
            cmox_ecdsa_verify(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.public_key.as_ptr(),
                self.public_key.len(),
                digest.as_ptr(),
                digest.len(),
                signature.as_bytes().as_ptr(),
                signature.as_bytes().len(),
                &mut fault_check,
            )
        };

        // Cleanup ECC context
        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        // Check result and fault check
        EccResult::from_rv(result)?;

        // Additional fault check - both result and fault_check must indicate success
        if result != fault_check {
            return Err(crate::error::EccError::BadParameter.into());
        }

        Ok(())
    }
}
