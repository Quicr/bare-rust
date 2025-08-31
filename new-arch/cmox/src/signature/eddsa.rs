use crate::ensure_initialized;
use crate::error::{EccError, EccResult, FromRetval};
use cmox_sys::*;
use core::mem::MaybeUninit;
use heapless::Vec;

/// Supported Edwards curves for EdDSA
#[derive(Copy, Clone, Debug)]
pub enum EdwardsCurve {
    /// Ed25519 curve - EdDSA
    Ed25519,
    /// Ed448 curve - EdDSA  
    Ed448,
}

impl EdwardsCurve {
    fn to_cmox_impl(self) -> cmox_ecc_impl_t {
        match self {
            EdwardsCurve::Ed25519 => unsafe { CMOX_ECC_ED25519_OPT_LOWMEM },
            EdwardsCurve::Ed448 => unsafe { CMOX_ECC_ED448_LOWMEM },
        }
    }

    fn math_funcs(self) -> cmox_math_funcs_t {
        match self {
            EdwardsCurve::Ed25519 => unsafe { CMOX_MATH_FUNCS_FAST },
            EdwardsCurve::Ed448 => unsafe { CMOX_MATH_FUNCS_FAST },
        }
    }

    fn signature_len(self) -> usize {
        match self {
            EdwardsCurve::Ed25519 => 64, // Ed25519 signatures are 64 bytes
            EdwardsCurve::Ed448 => 114,  // Ed448 signatures are 114 bytes
        }
    }

    fn private_key_len(self) -> usize {
        match self {
            EdwardsCurve::Ed25519 => 64, // Ed25519 private key is 64 bytes (secret + public)
            EdwardsCurve::Ed448 => 114,  // Ed448 private key is 114 bytes
        }
    }

    fn public_key_len(self) -> usize {
        match self {
            EdwardsCurve::Ed25519 => 32, // Ed25519 public key is 32 bytes
            EdwardsCurve::Ed448 => 57,   // Ed448 public key is 57 bytes
        }
    }
}

/// EdDSA signature with fixed-size components (max Ed448 size)
#[derive(Clone, Debug)]
pub struct EddsaSignature(Vec<u8, 56>);

impl EddsaSignature {
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

/// EdDSA signing operations
pub struct EddsaSigningKey {
    curve: EdwardsCurve,
    private_key: Vec<u8, 56>, // Max private key size for Ed448
}

impl EddsaSigningKey {
    /// Create a new EdDSA signing key
    pub fn new(curve: EdwardsCurve, bytes: &[u8]) -> crate::Result<Self> {
        let mut private_key = Vec::new();
        private_key
            .extend_from_slice(bytes)
            .map_err(|_| EccError::BadParameter)?;

        Ok(Self { curve, private_key })
    }

    /// Sign a message using EdDSA (note: EdDSA signs the full message, not just a digest)
    pub fn sign_message(&self, message: &[u8]) -> crate::Result<EddsaSignature> {
        ensure_initialized()?;

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

        let mut sig_len = self.curve.signature_len();
        let mut signature = Vec::new();
        let _ = signature.resize(sig_len, 0);

        // Call CMOX EdDSA sign
        let result = unsafe {
            cmox_eddsa_sign(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.private_key.as_ptr(),
                self.private_key.len(),
                message.as_ptr(),
                message.len(),
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

        Ok(EddsaSignature(signature))
    }
}

/// EdDSA verification operations
pub struct EddsaVerifyingKey {
    curve: EdwardsCurve,
    public_key: Vec<u8, 56>, // Max public key size for Ed448
}

impl EddsaVerifyingKey {
    /// Create a new EdDSA verifying key
    pub fn new(curve: EdwardsCurve, bytes: &[u8]) -> crate::Result<Self> {
        let mut public_key = Vec::new();
        public_key
            .extend_from_slice(bytes)
            .map_err(|_| EccError::BadParameter)?;

        Ok(Self { curve, public_key })
    }

    /// Verify a signature against a message
    pub fn verify_message(&self, message: &[u8], signature: &EddsaSignature) -> crate::Result<()> {
        ensure_initialized()?;

        // Check signature length
        if signature.0.len() != self.curve.signature_len() {
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

        // Call CMOX EdDSA verify
        let result = unsafe {
            cmox_eddsa_verify(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.public_key.as_ptr(),
                self.public_key.len(),
                message.as_ptr(),
                message.len(),
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
