//! Digital signature implementations using CMOX library
//!
//! This module provides implementations of digital signature algorithms using the
//! STM32 CMOX library. All operations (ECDSA, EdDSA, and RSA) use real cryptographic
//! implementations via their respective CMOX APIs.
//!
//! ## Features
//!
//! - **Real ECDSA**: Full implementation using CMOX ECC library
//!   - Supports P-256, P-384, P-521 curves
//!   - Real cryptographic signing and verification
//!   - Proper memory management and fault checking
//!
//! - **Real EdDSA**: Full implementation using CMOX ECC library
//!   - Supports Ed25519, Ed448 curves
//!   - Real cryptographic signing and verification of full messages
//!   - Deterministic signatures with internal hashing
//!   - Proper memory management and fault checking
//!
//! - **Real RSA**: Full implementation using CMOX RSA library
//!   - RSA-2048, RSA-4096 key sizes supported
//!   - PKCS#1 v1.5 signature scheme
//!   - SHA-256, SHA-384, SHA-512 hash algorithms
//!   - Real cryptographic operations with fault checking
//!
//! - **Real SM2**: Full implementation using CMOX SM2 library
//!   - SM2 curve (Chinese national cryptographic standard)
//!   - SM2 test curve for validation
//!   - ZA computation for user identity integration
//!   - Real cryptographic operations with fault checking

use crate::ensure_initialized;
use crate::error::{EccResult, FromRetval, RsaResult};
use crate::hash::sm3::Sm3;
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;
use digest::Digest;
use rand_core::{CryptoRng, RngCore};

/// Supported elliptic curves
#[derive(Copy, Clone, Debug)]
pub enum CurveType {
    /// NIST P-256 curve (secp256r1) - ECDSA
    P256,
    /// NIST P-384 curve (secp384r1) - ECDSA
    P384,
    /// NIST P-521 curve (secp521r1) - ECDSA
    P521,
}

/// Supported SM2 curves
#[derive(Copy, Clone, Debug)]
pub enum Sm2Curve {
    /// SM2 curve (production curve)
    Sm2,
    /// SM2 test curve (for testing)
    Sm2Test,
}

/// Supported Edwards curves for EdDSA
#[derive(Copy, Clone, Debug)]
pub enum EdwardsCurve {
    /// Ed25519 curve - EdDSA
    Ed25519,
    /// Ed448 curve - EdDSA  
    Ed448,
}

impl CurveType {
    fn to_cmox_impl(self) -> cmox_ecc_impl_t {
        match self {
            CurveType::P256 => unsafe { CMOX_ECC_SECP256R1_LOWMEM },
            CurveType::P384 => unsafe { CMOX_ECC_SECP384R1_LOWMEM },
            CurveType::P521 => unsafe { CMOX_ECC_SECP521R1_LOWMEM },
        }
    }

    fn math_funcs(self) -> cmox_math_funcs_t {
        match self {
            CurveType::P256 => unsafe { CMOX_MATH_FUNCS_SUPERFAST256 },
            CurveType::P384 => unsafe { CMOX_MATH_FUNCS_FAST },
            CurveType::P521 => unsafe { CMOX_MATH_FUNCS_FAST },
        }
    }

    fn signature_len(self) -> usize {
        match self {
            CurveType::P256 => 64,  // 2 * 32 bytes (r + s)
            CurveType::P384 => 96,  // 2 * 48 bytes
            CurveType::P521 => 132, // 2 * 66 bytes
        }
    }

    fn private_key_len(self) -> usize {
        match self {
            CurveType::P256 => 32,
            CurveType::P384 => 48,
            CurveType::P521 => 66,
        }
    }

    fn public_key_len(self) -> usize {
        match self {
            CurveType::P256 => 64,  // 2 * 32 bytes (x + y)
            CurveType::P384 => 96,  // 2 * 48 bytes
            CurveType::P521 => 132, // 2 * 66 bytes
        }
    }
}

impl Sm2Curve {
    fn to_cmox_impl(self) -> cmox_ecc_impl_t {
        match self {
            Sm2Curve::Sm2 => unsafe { CMOX_ECC_SM2_LOWMEM },
            Sm2Curve::Sm2Test => unsafe { CMOX_ECC_SM2TEST_LOWMEM },
        }
    }

    fn math_funcs(self) -> cmox_math_funcs_t {
        unsafe { CMOX_MATH_FUNCS_SUPERFAST256 } // SM2 is a 256-bit curve
    }

    /// Get the signature length in bytes for this SM2 curve
    pub fn signature_len(self) -> usize {
        64 // SM2 signatures are 64 bytes (r + s, 32 bytes each)
    }

    /// Get the private key length in bytes for this SM2 curve
    pub fn private_key_len(self) -> usize {
        32 // SM2 private key is 32 bytes
    }

    /// Get the public key length in bytes for this SM2 curve
    pub fn public_key_len(self) -> usize {
        64 // SM2 public key is 64 bytes (x + y, 32 bytes each)
    }

    /// Get the ZA value length in bytes for this SM2 curve
    pub fn za_len(self) -> usize {
        32 // ZA value is 32 bytes (SHA-256 output)
    }
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

/// ECDSA signature with fixed-size components (max P-521 size)
#[derive(Clone, Debug)]
pub struct EcdsaSignature {
    signature: [u8; 132], // Max signature size for P-521
    len: usize,
}

impl EcdsaSignature {
    /// Create signature from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() > 132 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut signature = [0u8; 132];
        signature[..bytes.len()].copy_from_slice(bytes);

        Ok(Self {
            signature,
            len: bytes.len(),
        })
    }

    /// Get signature as bytes  
    pub fn to_bytes(&self) -> &[u8] {
        &self.signature[..self.len]
    }

    /// Get signature length
    pub fn len(&self) -> usize {
        self.len
    }
}

/// ECDSA signing operations
pub struct EcdsaSigningKey {
    curve: CurveType,
    private_key: [u8; 66], // Max private key size for P-521
    private_key_len: usize,
}

impl EcdsaSigningKey {
    /// Create a new ECDSA signing key
    pub fn new(private_key: &[u8], curve: CurveType) -> crate::Result<Self> {
        if private_key.len() != curve.private_key_len() {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut key_buf = [0u8; 66];
        key_buf[..private_key.len()].copy_from_slice(private_key);

        Ok(Self {
            curve,
            private_key: key_buf,
            private_key_len: private_key.len(),
        })
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

        let signature_len = self.curve.signature_len();
        let mut signature_buf = [0u8; 132];
        let mut sig_len = signature_len;

        // Generate cryptographically secure random bytes for k value
        let mut random_k = [0u8; 64];
        rng.fill_bytes(&mut random_k);

        // Call CMOX ECDSA sign
        let result = unsafe {
            cmox_ecdsa_sign(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                random_k.as_ptr(),
                self.private_key_len, // Use private key length as random length
                self.private_key.as_ptr(),
                self.private_key_len,
                digest.as_ptr(),
                digest.len(),
                signature_buf.as_mut_ptr(),
                &mut sig_len,
            )
        };

        // Cleanup ECC context
        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        // Check result
        EccResult::from_rv(result)?;

        Ok(EcdsaSignature {
            signature: signature_buf,
            len: sig_len,
        })
    }
}

/// ECDSA verification operations
pub struct EcdsaVerifyingKey {
    curve: CurveType,
    #[allow(dead_code)]
    public_key: [u8; 132], // Max public key size for P-521
    public_key_len: usize,
}

impl EcdsaVerifyingKey {
    /// Create a new ECDSA verifying key
    pub fn new(public_key: &[u8], curve: CurveType) -> crate::Result<Self> {
        if public_key.len() != curve.public_key_len() {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut key_buf = [0u8; 132];
        key_buf[..public_key.len()].copy_from_slice(public_key);

        Ok(Self {
            curve,
            public_key: key_buf,
            public_key_len: public_key.len(),
        })
    }

    /// Verify a signature against a message digest
    pub fn verify_digest(&self, digest: &[u8], signature: &EcdsaSignature) -> crate::Result<()> {
        ensure_initialized()?;

        // Check signature length
        if signature.len() != self.curve.signature_len() {
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
                self.public_key_len,
                digest.as_ptr(),
                digest.len(),
                signature.signature.as_ptr(),
                signature.len(),
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

/// EdDSA signature with fixed-size components (max Ed448 size)
#[derive(Clone, Debug)]
pub struct EddsaSignature {
    signature: [u8; 114], // Max signature size for Ed448
    len: usize,
}

impl EddsaSignature {
    /// Create signature from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() > 114 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut signature = [0u8; 114];
        signature[..bytes.len()].copy_from_slice(bytes);

        Ok(Self {
            signature,
            len: bytes.len(),
        })
    }

    /// Get signature as bytes  
    pub fn to_bytes(&self) -> &[u8] {
        &self.signature[..self.len]
    }

    /// Get signature length
    pub fn len(&self) -> usize {
        self.len
    }
}

/// EdDSA signing operations
pub struct EddsaSigningKey {
    curve: EdwardsCurve,
    private_key: [u8; 114], // Max private key size for Ed448
    private_key_len: usize,
}

impl EddsaSigningKey {
    /// Create a new EdDSA signing key
    pub fn new(private_key: &[u8], curve: EdwardsCurve) -> crate::Result<Self> {
        if private_key.len() != curve.private_key_len() {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut key_buf = [0u8; 114];
        key_buf[..private_key.len()].copy_from_slice(private_key);

        Ok(Self {
            curve,
            private_key: key_buf,
            private_key_len: private_key.len(),
        })
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

        let signature_len = self.curve.signature_len();
        let mut signature_buf = [0u8; 114];
        let mut sig_len = signature_len;

        // Call CMOX EdDSA sign
        let result = unsafe {
            cmox_eddsa_sign(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.private_key.as_ptr(),
                self.private_key_len,
                message.as_ptr(),
                message.len(),
                signature_buf.as_mut_ptr(),
                &mut sig_len,
            )
        };

        // Cleanup ECC context
        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        // Check result
        EccResult::from_rv(result)?;

        Ok(EddsaSignature {
            signature: signature_buf,
            len: sig_len,
        })
    }
}

/// EdDSA verification operations
pub struct EddsaVerifyingKey {
    curve: EdwardsCurve,
    #[allow(dead_code)]
    public_key: [u8; 57], // Max public key size for Ed448
    public_key_len: usize,
}

impl EddsaVerifyingKey {
    /// Create a new EdDSA verifying key
    pub fn new(public_key: &[u8], curve: EdwardsCurve) -> crate::Result<Self> {
        if public_key.len() != curve.public_key_len() {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut key_buf = [0u8; 57];
        key_buf[..public_key.len()].copy_from_slice(public_key);

        Ok(Self {
            curve,
            public_key: key_buf,
            public_key_len: public_key.len(),
        })
    }

    /// Verify a signature against a message
    pub fn verify_message(&self, message: &[u8], signature: &EddsaSignature) -> crate::Result<()> {
        ensure_initialized()?;

        // Check signature length
        if signature.len() != self.curve.signature_len() {
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
                self.public_key_len,
                message.as_ptr(),
                message.len(),
                signature.signature.as_ptr(),
                signature.len(),
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

/// SM2 signature with fixed-size components
#[derive(Clone, Debug)]
pub struct Sm2Signature {
    signature: [u8; 64], // SM2 signatures are always 64 bytes
    len: usize,
}

impl Sm2Signature {
    /// Create signature from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() != 64 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut signature = [0u8; 64];
        signature.copy_from_slice(bytes);

        Ok(Self { signature, len: 64 })
    }

    /// Get signature as bytes  
    pub fn to_bytes(&self) -> &[u8] {
        &self.signature[..self.len]
    }

    /// Get signature length
    pub fn len(&self) -> usize {
        self.len
    }
}

/// SM2 signing operations
pub struct Sm2SigningKey {
    curve: Sm2Curve,
    private_key: [u8; 32], // SM2 private key is 32 bytes
    public_key: [u8; 64],  // SM2 public key is 64 bytes
    user_id: [u8; 64],     // User ID for ZA computation (max 64 bytes)
    user_id_len: usize,
}

impl Sm2SigningKey {
    /// Create a new SM2 signing key
    pub fn new(
        private_key: &[u8],
        public_key: &[u8],
        curve: Sm2Curve,
        user_id: &[u8],
    ) -> crate::Result<Self> {
        if private_key.len() != 32 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        if public_key.len() != 64 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        if user_id.len() > 64 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut private_key_buf = [0u8; 32];
        let mut public_key_buf = [0u8; 64];
        let mut user_id_buf = [0u8; 64];

        private_key_buf.copy_from_slice(private_key);
        public_key_buf.copy_from_slice(public_key);
        user_id_buf[..user_id.len()].copy_from_slice(user_id);

        Ok(Self {
            curve,
            private_key: private_key_buf,
            public_key: public_key_buf,
            user_id: user_id_buf,
            user_id_len: user_id.len(),
        })
    }

    /// Compute ZA value for SM2 signing
    pub fn compute_za(&self) -> crate::Result<[u8; 32]> {
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

        let mut za = [0u8; 32];
        let mut za_len = za.len();

        // Call CMOX SM2 computeZA - ENTLA is user_id length in bits
        let entla = (self.user_id_len * 8) as u16;
        let result = unsafe {
            cmox_sm2_computeZA(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.user_id.as_ptr(),
                entla,
                self.public_key.as_ptr(),
                self.public_key.len(),
                za.as_mut_ptr(),
                &mut za_len,
            )
        };

        // Cleanup ECC context
        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        // Check result
        EccResult::from_rv(result)?;

        Ok(za)
    }

    /// Sign a message using SM2
    ///
    /// This function implements the full SM2 signature process:
    /// 1. Computes ZA value from user identity and curve parameters
    /// 2. Computes SM3(ZA || message) digest per SM2 specification
    /// 3. Signs the resulting digest using SM2 algorithm
    ///
    /// # Arguments
    /// * `message` - The message to sign
    /// * `rng` - Cryptographically secure random number generator for generating k value
    pub fn sign_message<R>(&self, message: &[u8], rng: &mut R) -> crate::Result<Sm2Signature>
    where
        R: RngCore + CryptoRng,
    {
        ensure_initialized()?;

        // First compute ZA
        let za = self.compute_za()?;

        // Compute SM3 hash of ZA || message (per SM2 specification)
        let mut hasher = Sm3::new();
        Digest::update(&mut hasher, za); // Add ZA value
        Digest::update(&mut hasher, message); // Add message
        let digest_output = hasher.finalize();

        // Convert to byte array for signing
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&digest_output);

        self.sign_digest(&digest, rng)
    }

    /// Sign a pre-computed digest using SM2
    ///
    /// # Arguments
    /// * `digest` - The message digest to sign
    /// * `rng` - Cryptographically secure random number generator for generating k value
    pub fn sign_digest<R>(&self, digest: &[u8], rng: &mut R) -> crate::Result<Sm2Signature>
    where
        R: RngCore + CryptoRng,
    {
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

        let mut signature_buf = [0u8; 64];
        let mut sig_len = 64;

        // Generate cryptographically secure random bytes for k value
        let mut random_k = [0u8; 32];
        rng.fill_bytes(&mut random_k);

        // Call CMOX SM2 sign
        let result = unsafe {
            cmox_sm2_sign(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                random_k.as_ptr(),
                random_k.len(),
                self.private_key.as_ptr(),
                self.private_key.len(),
                digest.as_ptr(),
                digest.len(),
                signature_buf.as_mut_ptr(),
                &mut sig_len,
            )
        };

        // Cleanup ECC context
        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        // Check result
        EccResult::from_rv(result)?;

        Ok(Sm2Signature {
            signature: signature_buf,
            len: sig_len,
        })
    }
}

/// SM2 verification operations
pub struct Sm2VerifyingKey {
    curve: Sm2Curve,
    public_key: [u8; 64], // SM2 public key is 64 bytes
}

impl Sm2VerifyingKey {
    /// Create a new SM2 verifying key
    pub fn new(public_key: &[u8], curve: Sm2Curve) -> crate::Result<Self> {
        if public_key.len() != 64 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut key_buf = [0u8; 64];
        key_buf.copy_from_slice(public_key);

        Ok(Self {
            curve,
            public_key: key_buf,
        })
    }

    /// Verify a signature against a digest
    pub fn verify_digest(&self, digest: &[u8], signature: &Sm2Signature) -> crate::Result<()> {
        ensure_initialized()?;

        // Check signature length
        if signature.len() != 64 {
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

        // Call CMOX SM2 verify
        let result = unsafe {
            cmox_sm2_verify(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.public_key.as_ptr(),
                self.public_key.len(),
                digest.as_ptr(),
                digest.len(),
                signature.signature.as_ptr(),
                signature.len(),
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

/// RSA signature using PKCS#1 v1.5 signature scheme
#[derive(Clone, Debug)]
pub struct RsaSignature {
    signature: [u8; 512], // Max RSA-4096 signature size
    len: usize,
}

impl RsaSignature {
    /// Create signature from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() > 512 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut signature = [0u8; 512];
        signature[..bytes.len()].copy_from_slice(bytes);

        Ok(Self {
            signature,
            len: bytes.len(),
        })
    }

    /// Get signature as bytes
    pub fn to_bytes(&self) -> &[u8] {
        &self.signature[..self.len]
    }

    /// Get signature length
    pub fn len(&self) -> usize {
        self.len
    }
}

/// RSA key size enumeration
#[derive(Copy, Clone, Debug)]
pub enum RsaKeySize {
    /// RSA-2048 key size
    Rsa2048,
    /// RSA-4096 key size
    Rsa4096,
}

impl RsaKeySize {
    /// Get the key size in bits
    pub fn bit_size(self) -> usize {
        match self {
            RsaKeySize::Rsa2048 => 2048,
            RsaKeySize::Rsa4096 => 4096,
        }
    }

    fn signature_len(self) -> usize {
        match self {
            RsaKeySize::Rsa2048 => 256, // 2048 bits = 256 bytes
            RsaKeySize::Rsa4096 => 512, // 4096 bits = 512 bytes
        }
    }
}

/// Hash algorithm for RSA PKCS#1 v1.5 signatures
#[derive(Copy, Clone, Debug)]
pub enum RsaHashAlgorithm {
    /// SHA-256
    Sha256,
    /// SHA-384
    Sha384,
    /// SHA-512
    Sha512,
}

impl RsaHashAlgorithm {
    fn to_cmox_hash(self) -> cmox_rsa_pkcs1v15_hash_t {
        match self {
            RsaHashAlgorithm::Sha256 => unsafe { CMOX_RSA_PKCS1V15_HASH_SHA256 },
            RsaHashAlgorithm::Sha384 => unsafe { CMOX_RSA_PKCS1V15_HASH_SHA384 },
            RsaHashAlgorithm::Sha512 => unsafe { CMOX_RSA_PKCS1V15_HASH_SHA512 },
        }
    }
}

/// RSA signing key
pub struct RsaSigningKey {
    key_size: RsaKeySize,
    private_key: [u8; 512], // Max size for RSA-4096 private key
    private_key_len: usize,
    modulus: [u8; 512], // Max size for RSA-4096 modulus
    modulus_len: usize,
}

impl RsaSigningKey {
    /// Create new RSA signing key from modulus and private exponent
    pub fn new(
        modulus: &[u8],
        private_exponent: &[u8],
        key_size: RsaKeySize,
    ) -> crate::Result<Self> {
        ensure_initialized()?;

        let expected_mod_len = key_size.signature_len();
        if modulus.len() != expected_mod_len {
            return Err(crate::error::CoreError::InitFail.into());
        }

        if private_exponent.len() > expected_mod_len {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut mod_buf = [0u8; 512];
        let mut key_buf = [0u8; 512];

        mod_buf[..modulus.len()].copy_from_slice(modulus);
        key_buf[..private_exponent.len()].copy_from_slice(private_exponent);

        Ok(Self {
            key_size,
            private_key: key_buf,
            private_key_len: private_exponent.len(),
            modulus: mod_buf,
            modulus_len: modulus.len(),
        })
    }

    /// Sign a digest using RSA PKCS#1 v1.5
    pub fn sign_digest(
        &self,
        digest: &[u8],
        hash_alg: RsaHashAlgorithm,
    ) -> crate::Result<RsaSignature> {
        ensure_initialized()?;

        // Create RSA context with working buffer
        let mut working_buffer = [0u8; 4096]; // Larger buffer for RSA operations
        let mut rsa_ctx: cmox_rsa_handle_t = unsafe { MaybeUninit::zeroed().assume_init() };

        // Construct RSA context
        unsafe {
            cmox_rsa_construct(
                &mut rsa_ctx,
                CMOX_MATH_FUNCS_FAST,
                CMOX_MODEXP_PRIVATE_LOWMEM,
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );
        }

        // Set up RSA key
        let mut rsa_key: cmox_rsa_key_t = unsafe { MaybeUninit::zeroed().assume_init() };
        let result = unsafe {
            cmox_rsa_setKey(
                &mut rsa_key,
                self.modulus.as_ptr(),
                self.modulus_len,
                self.private_key.as_ptr(),
                self.private_key_len,
            )
        };

        if result != 0x00050000 {
            // CMOX_RSA_SUCCESS
            unsafe {
                cmox_rsa_cleanup(&mut rsa_ctx);
            }
            RsaResult::from_rv(result)?;
            return Err(crate::error::RsaError::Internal.into());
        }

        let signature_len = self.key_size.signature_len();
        let mut signature_buf = [0u8; 512];
        let mut sig_len = signature_len;

        // Call CMOX RSA PKCS#1 v1.5 sign
        let result = unsafe {
            cmox_rsa_pkcs1v15_sign(
                &mut rsa_ctx,
                &rsa_key,
                digest.as_ptr(),
                hash_alg.to_cmox_hash(),
                signature_buf.as_mut_ptr(),
                &mut sig_len,
            )
        };

        // Cleanup RSA context
        unsafe {
            cmox_rsa_cleanup(&mut rsa_ctx);
        }

        // Check result
        RsaResult::from_rv(result)?;

        Ok(RsaSignature {
            signature: signature_buf,
            len: sig_len,
        })
    }
}

/// RSA verifying key
pub struct RsaVerifyingKey {
    key_size: RsaKeySize,
    modulus: [u8; 512], // Max size for RSA-4096 modulus
    modulus_len: usize,
    public_exponent: [u8; 8], // Public exponent is typically small (e.g., 65537)
    public_exponent_len: usize,
}

impl RsaVerifyingKey {
    /// Create new RSA verifying key from modulus and public exponent
    pub fn new(
        modulus: &[u8],
        public_exponent: &[u8],
        key_size: RsaKeySize,
    ) -> crate::Result<Self> {
        ensure_initialized()?;

        let expected_mod_len = key_size.signature_len();
        if modulus.len() != expected_mod_len {
            return Err(crate::error::CoreError::InitFail.into());
        }

        if public_exponent.len() > 8 {
            return Err(crate::error::CoreError::InitFail.into());
        }

        let mut mod_buf = [0u8; 512];
        let mut exp_buf = [0u8; 8];

        mod_buf[..modulus.len()].copy_from_slice(modulus);
        exp_buf[..public_exponent.len()].copy_from_slice(public_exponent);

        Ok(Self {
            key_size,
            modulus: mod_buf,
            modulus_len: modulus.len(),
            public_exponent: exp_buf,
            public_exponent_len: public_exponent.len(),
        })
    }

    /// Verify a signature using RSA PKCS#1 v1.5
    pub fn verify_digest(
        &self,
        digest: &[u8],
        signature: &RsaSignature,
        hash_alg: RsaHashAlgorithm,
    ) -> crate::Result<()> {
        ensure_initialized()?;

        // Check signature length
        let expected_len = self.key_size.signature_len();
        if signature.len() != expected_len {
            return Err(crate::error::RsaError::BadParameter.into());
        }

        // Create RSA context with working buffer
        let mut working_buffer = [0u8; 4096]; // Larger buffer for RSA operations
        let mut rsa_ctx: cmox_rsa_handle_t = unsafe { MaybeUninit::zeroed().assume_init() };

        // Construct RSA context
        unsafe {
            cmox_rsa_construct(
                &mut rsa_ctx,
                CMOX_MATH_FUNCS_FAST,
                CMOX_MODEXP_PUBLIC,
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );
        }

        // Set up RSA public key
        let mut rsa_key: cmox_rsa_key_t = unsafe { MaybeUninit::zeroed().assume_init() };
        let result = unsafe {
            cmox_rsa_setKey(
                &mut rsa_key,
                self.modulus.as_ptr(),
                self.modulus_len,
                self.public_exponent.as_ptr(),
                self.public_exponent_len,
            )
        };

        if result != 0x00050000 {
            // CMOX_RSA_SUCCESS
            unsafe {
                cmox_rsa_cleanup(&mut rsa_ctx);
            }
            RsaResult::from_rv(result)?;
            return Err(crate::error::RsaError::Internal.into());
        }

        let mut fault_check: u32 = 0;

        // Call CMOX RSA PKCS#1 v1.5 verify
        let result = unsafe {
            cmox_rsa_pkcs1v15_verify(
                &mut rsa_ctx,
                &rsa_key,
                digest.as_ptr(),
                hash_alg.to_cmox_hash(),
                signature.signature.as_ptr(),
                signature.len(),
                &mut fault_check,
            )
        };

        // Cleanup RSA context
        unsafe {
            cmox_rsa_cleanup(&mut rsa_ctx);
        }

        // Check result and fault check
        RsaResult::from_rv(result)?;

        // Additional fault check - both result and fault_check must indicate success
        if result != fault_check {
            return Err(crate::error::RsaError::BadParameter.into());
        }

        Ok(())
    }
}

impl fmt::Debug for EcdsaSigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EcdsaSigningKey")
            .field("curve", &self.curve)
            .field("private_key_len", &self.private_key_len)
            .finish()
    }
}

impl fmt::Debug for EcdsaVerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EcdsaVerifyingKey")
            .field("curve", &self.curve)
            .field("public_key_len", &self.public_key_len)
            .finish()
    }
}

impl fmt::Debug for RsaSigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RsaSigningKey")
            .field("key_size", &self.key_size)
            .field("private_key_len", &self.private_key_len)
            .field("modulus_len", &self.modulus_len)
            .finish()
    }
}

impl fmt::Debug for RsaVerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RsaVerifyingKey")
            .field("key_size", &self.key_size)
            .field("modulus_len", &self.modulus_len)
            .field("public_exponent_len", &self.public_exponent_len)
            .finish()
    }
}

impl fmt::Debug for EddsaSigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EddsaSigningKey")
            .field("curve", &self.curve)
            .field("private_key_len", &self.private_key_len)
            .finish()
    }
}

impl fmt::Debug for EddsaVerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EddsaVerifyingKey")
            .field("curve", &self.curve)
            .field("public_key_len", &self.public_key_len)
            .finish()
    }
}

impl fmt::Debug for Sm2SigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sm2SigningKey")
            .field("curve", &self.curve)
            .field("user_id_len", &self.user_id_len)
            .finish()
    }
}

impl fmt::Debug for Sm2VerifyingKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sm2VerifyingKey")
            .field("curve", &self.curve)
            .finish()
    }
}
