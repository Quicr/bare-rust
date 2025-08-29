//! ECDH key exchange examples using CMOX library
//!
//! This example demonstrates ECDH key exchange with both NIST curves and Montgomery curves.

#![no_std]
#![no_main]

use cmox::{
    ecdh::{EcdhCurve, EcdhKeyPair, EcdhPrivateKey, EcdhPublicKey},
    initialize, CmoxError, EccError,
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().unwrap();

    // Run ECDH examples
    ecdh_example();
    ecdh_usage_patterns();
    ecdh_error_handling_example();
}

/// Example using ECDH with NIST curves (P-256, P-384, P-521)
fn ecdh_example() {
    let curves = [
        EcdhCurve::P256,
        EcdhCurve::P384,
        EcdhCurve::P521,
        EcdhCurve::X25519,
        EcdhCurve::X448,
    ];

    for curve in curves {
        let alice_random = [0x01; 32]; // Alice's random data
        let bob_random = [0x02; 32]; // Bob's random data

        // Generate key pairs for Alice and Bob
        let alice_keypair = EcdhKeyPair::generate(EcdhCurve::P256, &alice_random).unwrap();
        let bob_keypair = EcdhKeyPair::generate(EcdhCurve::P256, &bob_random).unwrap();

        // Perform ECDH on both sides
        let alice_shared = alice_keypair
            .private_key
            .exchange(&bob_keypair.public_key)
            .unwrap();
        let bob_shared = bob_keypair
            .private_key
            .exchange(&alice_keypair.public_key)
            .unwrap();

        // Shared secrets should be identical
        assert!(alice_shared.as_bytes() == bob_shared.as_bytes());
        assert!(alice_shared.as_bytes().len() == curve.shared_secret_len());
    }
}

/// Example showing different ECDH usage patterns
fn ecdh_usage_patterns() {
    // Pattern 1: Key exchange with serialized keys
    {
        let alice_random = [0x10; 32];
        let bob_random = [0x11; 32];

        let alice_keypair = EcdhKeyPair::generate(EcdhCurve::X25519, &alice_random).unwrap();
        let bob_keypair = EcdhKeyPair::generate(EcdhCurve::X25519, &bob_random).unwrap();

        // Alice sends her public key to Bob (and vice versa)
        let alice_public_bytes = alice_keypair.public_key.as_bytes();
        let bob_public_bytes = bob_keypair.public_key.as_bytes();

        // Reconstruct public keys from bytes
        let alice_public =
            EcdhPublicKey::from_bytes(EcdhCurve::X25519, alice_public_bytes).unwrap();
        let bob_public = EcdhPublicKey::from_bytes(EcdhCurve::X25519, bob_public_bytes).unwrap();

        // Perform key exchange
        if let (Ok(alice_shared), Ok(bob_shared)) = (
            alice_keypair.private_key.exchange(&bob_public),
            bob_keypair.private_key.exchange(&alice_public),
        ) {
            // Both parties now have the same shared secret
            assert!(alice_shared.as_bytes() == bob_shared.as_bytes());
        }
    }

    // Pattern 3: Cross-curve validation (should fail)
    {
        let alice_random = [0x13; 32];
        let bob_random = [0x14; 48];

        let alice_keypair = EcdhKeyPair::generate(EcdhCurve::X25519, &alice_random).unwrap();
        let bob_keypair = EcdhKeyPair::generate(EcdhCurve::P256, &bob_random).unwrap();

        // This should fail - different curves
        match alice_keypair.private_key.exchange(&bob_keypair.public_key) {
            Err(CmoxError::Ecc(EccError::BadParameter)) => {
                // Expected error for mismatched curves
            }
            _ => assert!(false, "Operation should have failed"),
        }
    }
}

/// Example showing ECDH error handling patterns
fn ecdh_error_handling_example() {
    // Test insufficient random data
    {
        let short_random = [0x20; 16]; // Too short for any curve

        match EcdhKeyPair::generate(EcdhCurve::P256, &short_random) {
            Err(CmoxError::Ecc(EccError::WrongRandom)) => {
                // Expected error for insufficient random data
            }
            _ => assert!(false, "Operation should have failed"),
        }
    }

    // Test invalid key sizes
    {
        let wrong_private_key = [0x21; 31]; // Wrong size for P-256 (should be 32)
        let wrong_public_key = [0x22; 63]; // Wrong size for P-256 (should be 65)

        match EcdhPrivateKey::from_bytes(EcdhCurve::P256, &wrong_private_key) {
            Err(CmoxError::Ecc(EccError::BadParameter)) => {
                // Expected error for wrong key size
            }
            _ => assert!(false, "Operation should have failed"),
        }

        match EcdhPublicKey::from_bytes(EcdhCurve::P256, &wrong_public_key) {
            Err(CmoxError::Ecc(EccError::BadParameter)) => {
                // Expected error for wrong key size
            }
            _ => assert!(false, "Operation should have failed"),
        }
    }

    // Test curve mismatch
    {
        let alice_random = [0x23; 32];
        let bob_random = [0x24; 32];

        let alice_keypair = EcdhKeyPair::generate(EcdhCurve::P256, &alice_random).unwrap();
        let bob_keypair = EcdhKeyPair::generate(EcdhCurve::X25519, &bob_random).unwrap();

        // Try to perform ECDH with mismatched curves
        match alice_keypair.private_key.exchange(&bob_keypair.public_key) {
            Err(CmoxError::Ecc(EccError::BadParameter)) => {
                // Expected error for curve mismatch
            }
            _ => assert!(false, "Operation should have failed"),
        }
    }
}
