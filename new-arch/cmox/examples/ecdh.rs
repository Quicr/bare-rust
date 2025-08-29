//! ECDH key exchange examples using CMOX library
//!
//! This example demonstrates ECDH key exchange with both NIST curves and Montgomery curves.

#![no_std]
#![no_main]

use cmox::{
    ecdh::{EcdhCurve, EcdhKeyPair, EcdhPrivateKey, EcdhPublicKey},
    initialize, CmoxError,
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // Run ECDH examples
    nist_curves_example();
    montgomery_curves_example();
    ecdh_usage_patterns();
    ecdh_error_handling_example();
}

/// Example using ECDH with NIST curves (P-256, P-384, P-521)
fn nist_curves_example() {
    // ECDH P-256 Example
    {
        let alice_random = [0x01; 32]; // Alice's random data
        let bob_random = [0x02; 32]; // Bob's random data

        // Generate key pairs for Alice and Bob
        match (
            EcdhKeyPair::generate(EcdhCurve::P256, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::P256, &bob_random),
        ) {
            (Ok(alice_keypair), Ok(bob_keypair)) => {
                // Extract public keys
                let alice_public = alice_keypair.public_key();
                let bob_public = bob_keypair.public_key();

                // Perform ECDH on both sides
                match (
                    alice_keypair.private_key().exchange(bob_public),
                    bob_keypair.private_key().exchange(alice_public),
                ) {
                    (Ok(alice_shared), Ok(bob_shared)) => {
                        // Shared secrets should be identical
                        assert!(alice_shared.to_bytes() == bob_shared.to_bytes());
                        assert!(alice_shared.len() == 32); // P-256 shared secret is 32 bytes

                        // ECDH successful
                    }
                    _ => {
                        // ECDH computation failed
                    }
                }
            }
            _ => {
                // Key generation failed
            }
        }
    }

    // ECDH P-384 Example
    {
        let alice_random = [0x03; 48]; // Alice's random data (48 bytes for P-384)
        let bob_random = [0x04; 48]; // Bob's random data

        match (
            EcdhKeyPair::generate(EcdhCurve::P384, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::P384, &bob_random),
        ) {
            (Ok(alice_keypair), Ok(bob_keypair)) => {
                match (
                    alice_keypair
                        .private_key()
                        .exchange(bob_keypair.public_key()),
                    bob_keypair
                        .private_key()
                        .exchange(alice_keypair.public_key()),
                ) {
                    (Ok(alice_shared), Ok(bob_shared)) => {
                        assert!(alice_shared.to_bytes() == bob_shared.to_bytes());
                        assert!(alice_shared.len() == 48); // P-384 shared secret is 48 bytes
                    }
                    _ => {
                        // ECDH computation failed
                    }
                }
            }
            _ => {
                // Key generation failed
            }
        }
    }

    // ECDH P-521 Example
    {
        let alice_random = [0x05; 66]; // Alice's random data (66 bytes for P-521)
        let bob_random = [0x06; 66]; // Bob's random data

        match (
            EcdhKeyPair::generate(EcdhCurve::P521, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::P521, &bob_random),
        ) {
            (Ok(alice_keypair), Ok(bob_keypair)) => {
                match (
                    alice_keypair
                        .private_key()
                        .exchange(bob_keypair.public_key()),
                    bob_keypair
                        .private_key()
                        .exchange(alice_keypair.public_key()),
                ) {
                    (Ok(alice_shared), Ok(bob_shared)) => {
                        assert!(alice_shared.to_bytes() == bob_shared.to_bytes());
                        assert!(alice_shared.len() == 66); // P-521 shared secret is 66 bytes
                    }
                    _ => {
                        // ECDH computation failed
                    }
                }
            }
            _ => {
                // Key generation failed
            }
        }
    }
}

/// Example using ECDH with Montgomery curves (X25519, X448)
fn montgomery_curves_example() {
    // X25519 Example
    {
        let alice_random = [0x07; 32]; // Alice's random data
        let bob_random = [0x08; 32]; // Bob's random data

        // Generate key pairs
        match (
            EcdhKeyPair::generate(EcdhCurve::X25519, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::X25519, &bob_random),
        ) {
            (Ok(alice_keypair), Ok(bob_keypair)) => {
                // X25519 keys are compact - 32-byte public keys vs 64-byte for P-256
                assert!(alice_keypair.public_key().to_bytes().len() == 32);
                assert!(bob_keypair.public_key().to_bytes().len() == 32);

                // Perform ECDH
                match (
                    alice_keypair
                        .private_key()
                        .exchange(bob_keypair.public_key()),
                    bob_keypair
                        .private_key()
                        .exchange(alice_keypair.public_key()),
                ) {
                    (Ok(alice_shared), Ok(bob_shared)) => {
                        // Shared secrets should be identical
                        assert!(alice_shared.to_bytes() == bob_shared.to_bytes());
                        assert!(alice_shared.len() == 32); // X25519 shared secret is 32 bytes

                        // X25519 ECDH successful
                    }
                    _ => {
                        // ECDH computation failed
                    }
                }
            }
            _ => {
                // Key generation failed
            }
        }
    }

    // X448 Example
    {
        let alice_random = [0x09; 56]; // Alice's random data (56 bytes for X448)
        let bob_random = [0x0A; 56]; // Bob's random data

        match (
            EcdhKeyPair::generate(EcdhCurve::X448, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::X448, &bob_random),
        ) {
            (Ok(alice_keypair), Ok(bob_keypair)) => {
                // X448 keys are 56 bytes
                assert!(alice_keypair.public_key().to_bytes().len() == 56);
                assert!(bob_keypair.public_key().to_bytes().len() == 56);

                match (
                    alice_keypair
                        .private_key()
                        .exchange(bob_keypair.public_key()),
                    bob_keypair
                        .private_key()
                        .exchange(alice_keypair.public_key()),
                ) {
                    (Ok(alice_shared), Ok(bob_shared)) => {
                        assert!(alice_shared.to_bytes() == bob_shared.to_bytes());
                        assert!(alice_shared.len() == 56); // X448 shared secret is 56 bytes
                    }
                    _ => {
                        // ECDH computation failed
                    }
                }
            }
            _ => {
                // Key generation failed
            }
        }
    }

    // Compare curve properties
    curve_properties_demo();
}

/// Demonstrate curve properties and usage patterns
fn curve_properties_demo() {
    // Demonstrate different curve properties
    let curves = [
        EcdhCurve::P256,
        EcdhCurve::P384,
        EcdhCurve::P521,
        EcdhCurve::X25519,
        EcdhCurve::X448,
    ];

    for curve in curves.iter() {
        // Check curve type
        if curve.is_weierstrass() {
            // NIST curves use uncompressed point representation (x,y coordinates)
            // Public key = x || y (both coordinates)
            // Compatible with ECDSA keys
        } else if curve.is_montgomery() {
            // Montgomery curves use compact representation (x-coordinate only)
            // Faster, constant-time operations
            // Designed specifically for ECDH
        }
    }
}

/// Example showing different ECDH usage patterns
fn ecdh_usage_patterns() {
    // Pattern 1: Key exchange with serialized keys
    {
        let alice_random = [0x10; 32];
        let bob_random = [0x11; 32];

        if let (Ok(alice_keypair), Ok(bob_keypair)) = (
            EcdhKeyPair::generate(EcdhCurve::X25519, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::X25519, &bob_random),
        ) {
            // Alice sends her public key to Bob (and vice versa)
            let alice_public_bytes = alice_keypair.public_key().to_bytes();
            let bob_public_bytes = bob_keypair.public_key().to_bytes();

            // Reconstruct public keys from bytes
            if let (Ok(alice_public), Ok(bob_public)) = (
                EcdhPublicKey::from_bytes(alice_public_bytes, EcdhCurve::X25519),
                EcdhPublicKey::from_bytes(bob_public_bytes, EcdhCurve::X25519),
            ) {
                // Perform key exchange
                if let (Ok(alice_shared), Ok(bob_shared)) = (
                    alice_keypair.private_key().exchange(&bob_public),
                    bob_keypair.private_key().exchange(&alice_public),
                ) {
                    // Both parties now have the same shared secret
                    assert!(alice_shared.to_bytes() == bob_shared.to_bytes());
                }
            }
        }
    }

    // Pattern 2: Using separate private/public keys
    {
        let alice_random = [0x12; 32];

        if let Ok(alice_keypair) = EcdhKeyPair::generate(EcdhCurve::X25519, &alice_random) {
            // Split into separate private and public keys
            let (alice_private, alice_public) = alice_keypair.into_keys();

            // Alice can use her private key for multiple exchanges
            // while sharing her public key with multiple parties

            // Store or use the keys separately
            let _private_key_bytes = alice_private.to_bytes();
            let _public_key_bytes = alice_public.to_bytes();
        }
    }

    // Pattern 3: Cross-curve validation (should fail)
    {
        let alice_random = [0x13; 32];
        let bob_random = [0x14; 48];

        if let (Ok(alice_keypair), Ok(bob_keypair)) = (
            EcdhKeyPair::generate(EcdhCurve::X25519, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::P384, &bob_random),
        ) {
            // This should fail - different curves
            match alice_keypair
                .private_key()
                .exchange(bob_keypair.public_key())
            {
                Ok(_) => {
                    // This should not happen
                }
                Err(CmoxError::InvalidInput) => {
                    // Expected error for mismatched curves
                }
                Err(_) => {
                    // Other error
                }
            }
        }
    }
}

/// Example showing ECDH error handling patterns
fn ecdh_error_handling_example() {
    // Test insufficient random data
    {
        let short_random = [0x20; 16]; // Too short for any curve

        match EcdhKeyPair::generate(EcdhCurve::P256, &short_random) {
            Ok(_) => {
                // This should not happen
            }
            Err(CmoxError::WrongRandom) => {
                // Expected error for insufficient random data
            }
            Err(_) => {
                // Other error
            }
        }
    }

    // Test invalid key sizes
    {
        let wrong_private_key = [0x21; 31]; // Wrong size for P-256 (should be 32)
        let wrong_public_key = [0x22; 63]; // Wrong size for P-256 (should be 64)

        match EcdhPrivateKey::from_bytes(&wrong_private_key, EcdhCurve::P256) {
            Ok(_) => {
                // This should not happen
            }
            Err(CmoxError::InvalidInput) => {
                // Expected error for wrong key size
            }
            Err(_) => {
                // Other error
            }
        }

        match EcdhPublicKey::from_bytes(&wrong_public_key, EcdhCurve::P256) {
            Ok(_) => {
                // This should not happen
            }
            Err(CmoxError::InvalidInput) => {
                // Expected error for wrong key size
            }
            Err(_) => {
                // Other error
            }
        }
    }

    // Test curve mismatch
    {
        let alice_random = [0x23; 32];
        let bob_random = [0x24; 32];

        if let (Ok(alice_keypair), Ok(bob_keypair)) = (
            EcdhKeyPair::generate(EcdhCurve::P256, &alice_random),
            EcdhKeyPair::generate(EcdhCurve::X25519, &bob_random),
        ) {
            // Try to perform ECDH with mismatched curves
            match alice_keypair
                .private_key()
                .exchange(bob_keypair.public_key())
            {
                Ok(_) => {
                    // This should not happen
                }
                Err(CmoxError::InvalidInput) => {
                    // Expected error for curve mismatch
                }
                Err(_) => {
                    // Other error
                }
            }
        }
    }
}
