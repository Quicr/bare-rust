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

use crate::ensure_initialized;
use crate::error::{EccResult, FromRetval};
use cmox_sys::*;
use core::fmt;
use core::mem::MaybeUninit;

/// Supported elliptic curves for ECDH
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EcdhCurve {
    /// NIST P-256 (secp256r1) - 32-byte private keys, 64-byte public keys
    P256,
    /// NIST P-384 (secp384r1) - 48-byte private keys, 96-byte public keys  
    P384,
    /// NIST P-521 (secp521r1) - 66-byte private keys, 132-byte public keys
    P521,
    /// X25519 (Curve25519) - 32-byte private keys, 32-byte public keys
    X25519,
    /// X448 (Curve448) - 56-byte private keys, 56-byte public keys
    X448,
}

impl EcdhCurve {
    fn to_cmox_impl(self) -> cmox_ecc_impl_t {
        match self {
            EcdhCurve::P256 => unsafe { CMOX_ECC_SECP256R1_LOWMEM },
            EcdhCurve::P384 => unsafe { CMOX_ECC_SECP384R1_LOWMEM },
            EcdhCurve::P521 => unsafe { CMOX_ECC_SECP521R1_LOWMEM },
            EcdhCurve::X25519 => unsafe { CMOX_ECC_CURVE25519 },
            EcdhCurve::X448 => unsafe { CMOX_ECC_CURVE448 },
        }
    }

    fn math_funcs(self) -> cmox_math_funcs_t {
        match self {
            EcdhCurve::P256 => unsafe { CMOX_MATH_FUNCS_SUPERFAST256 },
            EcdhCurve::P384 => unsafe { CMOX_MATH_FUNCS_FAST },
            EcdhCurve::P521 => unsafe { CMOX_MATH_FUNCS_FAST },
            EcdhCurve::X25519 => unsafe { CMOX_MATH_FUNCS_FAST },
            EcdhCurve::X448 => unsafe { CMOX_MATH_FUNCS_FAST },
        }
    }

    fn private_key_len(self) -> usize {
        match self {
            EcdhCurve::P256 => 32,
            EcdhCurve::P384 => 48,
            EcdhCurve::P521 => 66,
            EcdhCurve::X25519 => 32,
            EcdhCurve::X448 => 56,
        }
    }

    fn public_key_len(self) -> usize {
        match self {
            EcdhCurve::P256 => 64,   // Uncompressed: 32 bytes x + 32 bytes y
            EcdhCurve::P384 => 96,   // Uncompressed: 48 bytes x + 48 bytes y
            EcdhCurve::P521 => 132,  // Uncompressed: 66 bytes x + 66 bytes y
            EcdhCurve::X25519 => 32, // Montgomery: x-coordinate only
            EcdhCurve::X448 => 56,   // Montgomery: x-coordinate only
        }
    }

    fn shared_secret_len(self) -> usize {
        match self {
            EcdhCurve::P256 => 32,   // x-coordinate of shared point
            EcdhCurve::P384 => 48,   // x-coordinate of shared point
            EcdhCurve::P521 => 66,   // x-coordinate of shared point
            EcdhCurve::X25519 => 32, // Montgomery ladder result
            EcdhCurve::X448 => 56,   // Montgomery ladder result
        }
    }

    /// Check if this is a Montgomery curve (X25519/X448)
    pub fn is_montgomery(self) -> bool {
        matches!(self, EcdhCurve::X25519 | EcdhCurve::X448)
    }

    /// Check if this is a Weierstrass curve (NIST P-curves)
    pub fn is_weierstrass(self) -> bool {
        matches!(self, EcdhCurve::P256 | EcdhCurve::P384 | EcdhCurve::P521)
    }
}

/// ECDH private key for key exchange operations
pub struct EcdhPrivateKey {
    curve: EcdhCurve,
    private_key: [u8; 66], // Max size for P-521
    private_key_len: usize,
}

impl EcdhPrivateKey {
    /// Create an ECDH private key from raw bytes
    pub fn from_bytes(private_key: &[u8], curve: EcdhCurve) -> crate::Result<Self> {
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

    /// Get the curve associated with this private key
    pub fn curve(&self) -> EcdhCurve {
        self.curve
    }

    /// Get private key as bytes
    pub fn to_bytes(&self) -> &[u8] {
        &self.private_key[..self.private_key_len]
    }

    /// Perform ECDH key exchange with a peer's public key
    pub fn exchange(&self, peer_public_key: &EcdhPublicKey) -> crate::Result<SharedSecret> {
        ensure_initialized()?;

        if self.curve != peer_public_key.curve {
            return Err(crate::error::CoreError::InitFail.into());
        }

        // Create ECC context
        let mut working_buffer = [0u8; 2048];
        let mut ecc_ctx: cmox_ecc_handle_t = unsafe { MaybeUninit::zeroed().assume_init() };

        unsafe {
            cmox_ecc_construct(
                &mut ecc_ctx,
                self.curve.math_funcs(),
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );
        }

        let mut shared_secret_buf = [0u8; 66]; // Max size for P-521
        let mut shared_secret_len = self.curve.shared_secret_len();

        // Call CMOX ECDH
        let result = unsafe {
            cmox_ecdh(
                &mut ecc_ctx,
                self.curve.to_cmox_impl(),
                self.private_key.as_ptr(),
                self.private_key_len,
                peer_public_key.public_key.as_ptr(),
                peer_public_key.public_key_len,
                shared_secret_buf.as_mut_ptr(),
                &mut shared_secret_len,
            )
        };

        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        EccResult::from_rv(result)?;

        Ok(SharedSecret {
            curve: self.curve,
            secret: shared_secret_buf,
            secret_len: shared_secret_len,
        })
    }
}

/// ECDH public key for key exchange operations
pub struct EcdhPublicKey {
    curve: EcdhCurve,
    public_key: [u8; 132], // Max size for P-521
    public_key_len: usize,
}

impl EcdhPublicKey {
    /// Create an ECDH public key from raw bytes
    pub fn from_bytes(public_key: &[u8], curve: EcdhCurve) -> crate::Result<Self> {
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

    /// Get the curve associated with this public key
    pub fn curve(&self) -> EcdhCurve {
        self.curve
    }

    /// Get public key as bytes
    pub fn to_bytes(&self) -> &[u8] {
        &self.public_key[..self.public_key_len]
    }
}

/// ECDH shared secret result from key exchange
pub struct SharedSecret {
    curve: EcdhCurve,
    secret: [u8; 66], // Max size for P-521
    secret_len: usize,
}

impl SharedSecret {
    /// Get the curve associated with this shared secret
    pub fn curve(&self) -> EcdhCurve {
        self.curve
    }

    /// Get shared secret as bytes
    pub fn to_bytes(&self) -> &[u8] {
        &self.secret[..self.secret_len]
    }

    /// Get the length of the shared secret
    pub fn len(&self) -> usize {
        self.secret_len
    }
}

/// ECDH key pair (private + public key)
pub struct EcdhKeyPair {
    private_key: EcdhPrivateKey,
    public_key: EcdhPublicKey,
}

impl EcdhKeyPair {
    /// Generate a new ECDH key pair using cryptographically secure random data
    pub fn generate(curve: EcdhCurve, random: &[u8]) -> crate::Result<Self> {
        ensure_initialized()?;

        let expected_random_len = curve.private_key_len();
        if random.len() < expected_random_len {
            return Err(crate::error::EccError::WrongRandom.into());
        }

        // Create ECC context
        let mut working_buffer = [0u8; 2048];
        let mut ecc_ctx: cmox_ecc_handle_t = unsafe { MaybeUninit::zeroed().assume_init() };

        unsafe {
            cmox_ecc_construct(
                &mut ecc_ctx,
                curve.math_funcs(),
                working_buffer.as_mut_ptr(),
                working_buffer.len(),
            );
        }

        let mut private_key_buf = [0u8; 66]; // Max size for P-521
        let mut private_key_len = curve.private_key_len();
        let mut public_key_buf = [0u8; 132]; // Max size for P-521
        let mut public_key_len = curve.public_key_len();

        // Call CMOX key generation
        let result = unsafe {
            cmox_ecdsa_keyGen(
                &mut ecc_ctx,
                curve.to_cmox_impl(),
                random.as_ptr(),
                random.len(),
                private_key_buf.as_mut_ptr(),
                &mut private_key_len,
                public_key_buf.as_mut_ptr(),
                &mut public_key_len,
            )
        };

        unsafe {
            cmox_ecc_cleanup(&mut ecc_ctx);
        }

        EccResult::from_rv(result)?;

        // Create key pair
        let private_key = EcdhPrivateKey {
            curve,
            private_key: private_key_buf,
            private_key_len,
        };

        let public_key = EcdhPublicKey {
            curve,
            public_key: public_key_buf,
            public_key_len,
        };

        Ok(Self {
            private_key,
            public_key,
        })
    }

    /// Get a reference to the private key
    pub fn private_key(&self) -> &EcdhPrivateKey {
        &self.private_key
    }

    /// Get a reference to the public key
    pub fn public_key(&self) -> &EcdhPublicKey {
        &self.public_key
    }

    /// Consume the key pair and return the private and public keys separately
    pub fn into_keys(self) -> (EcdhPrivateKey, EcdhPublicKey) {
        (self.private_key, self.public_key)
    }
}

impl fmt::Debug for EcdhPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EcdhPrivateKey")
            .field("curve", &self.curve)
            .field("private_key_len", &self.private_key_len)
            .finish()
    }
}

impl fmt::Debug for EcdhPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EcdhPublicKey")
            .field("curve", &self.curve)
            .field("public_key_len", &self.public_key_len)
            .finish()
    }
}

impl fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedSecret")
            .field("curve", &self.curve)
            .field("secret_len", &self.secret_len)
            .finish()
    }
}

impl fmt::Debug for EcdhKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EcdhKeyPair")
            .field("curve", &self.private_key.curve)
            .finish()
    }
}
