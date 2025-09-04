//! Digital signature implementations using CMOX library
//!
//! This module provides implementations of digital signature algorithms using the
//! STM32 CMOX library.  The following signature algorithms are implemented:
//!
//! * ECDSA
//!     * P-256 with SHA-256
//!     * P-384 with SHA-384
//!     * P-521 with SHA-512
//! * EdDSA with Ed25519 and Ed448
//!
//! These functions are available using the Rust Crypto Signer and RandomizedSigner traits.
//!
//! RSA and SM2 signatures are not implemented.  The lack of a uniform signature API makes this
//! more labor-intensive, and ECDSA / EdDSA are a higher priority.

/// Elliptic Curve Digital Signature Algorithm
pub mod ecdsa;

/// Edwards Curve Digital Signature Algorithm
pub mod eddsa;
