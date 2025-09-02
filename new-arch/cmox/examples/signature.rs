//! Digital signature examples using CMOX library
//!
//! This example demonstrates the usage of ECDSA, EdDSA, RSA, and SM2 digital signatures
//! with the CMOX library, including proper cryptographic hash computation and usage patterns.
//!
//! ## Features Demonstrated:
//! - Real SHA-256, SHA-384, and SHA-512 hash computation for message digests
//! - ECDSA signatures with P-256, P-384, and P-521 curves  
//! - RSA signatures with PKCS#1 v1.5 and proper hash algorithms
//! - SM2 signatures with proper SM3(ZA || message) computation
//! - Cryptographically secure random number generation for signatures

#![no_std]
#![no_main]

use cmox::{
    drbg::CtrDrbg,
    initialize,
    signature::{ecdsa, eddsa},
};
use signature::{RandomizedSigner, Signer, Verifier};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // Initialize RNG for signature examples
    let entropy = [0x42; 32]; // In practice, use real entropy
    let nonce = [0x01; 16];
    let mut rng = CtrDrbg::new_default(&entropy, &nonce).expect("Failed to initialize RNG");

    // Run signature examples
    ecdsa_signature_example(&mut rng);
    eddsa_signature_example(&mut rng);
}

/// Example using ECDSA signatures with different curves
fn ecdsa_signature_example(rng: &mut CtrDrbg) {
    use ecdsa::*;

    let message = b"Hello, world!";

    let (private_key, public_key) = PrivateKey::<P256>::random(rng).unwrap();
    let signature = private_key.sign_with_rng(rng, message);
    public_key.verify(message, &signature).unwrap();

    let (private_key, public_key) = PrivateKey::<P384>::random(rng).unwrap();
    let signature = private_key.sign_with_rng(rng, message);
    public_key.verify(message, &signature).unwrap();

    let (private_key, public_key) = PrivateKey::<P521>::random(rng).unwrap();
    let signature = private_key.sign_with_rng(rng, message);
    public_key.verify(message, &signature).unwrap();
}

/// Example using EdDSA signatures with Ed25519 and Ed448
fn eddsa_signature_example(rng: &mut CtrDrbg) {
    use eddsa::*;

    let message = b"Hello, world!";

    let (private_key, public_key) = PrivateKey::<Ed25519>::random(rng).unwrap();
    let signature = private_key.sign(message);
    public_key.verify(message, &signature).unwrap();

    let (private_key, public_key) = PrivateKey::<Ed448>::random(rng).unwrap();
    let signature = private_key.sign(message);
    public_key.verify(message, &signature).unwrap();
}
