//! Digital signature examples using CMOX library
//! 
//! This example demonstrates the usage of ECDSA, EdDSA, RSA, and SM2 digital signatures
//! with the CMOX library, including error handling and usage patterns.

#![no_std]
#![no_main]

use cmox::{
    initialize,
    signature::{
        CurveType, EcdsaSigningKey, EcdsaVerifyingKey, EcdsaSignature,
        EdwardsCurve, EddsaSigningKey, EddsaVerifyingKey, EddsaSignature,
        RsaSigningKey, RsaVerifyingKey, RsaSignature, RsaKeySize, RsaHashAlgorithm,
        Sm2Curve, Sm2SigningKey, Sm2VerifyingKey, Sm2Signature
    },
    CmoxError
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // Run signature examples
    ecdsa_signature_example();
    eddsa_signature_example();
    rsa_signature_example();
    sm2_signature_example();
    signature_error_handling_example();
    signature_usage_patterns();
}

/// Example using ECDSA signatures with different curves
fn ecdsa_signature_example() {
    // ECDSA P-256 Example
    {
        let private_key = [0x01; 32]; // 256-bit private key
        let public_key = [0x02; 64];  // Uncompressed public key (x + y coordinates)
        
        // Create signing and verifying keys
        match (
            EcdsaSigningKey::new(&private_key, CurveType::P256),
            EcdsaVerifyingKey::new(&public_key, CurveType::P256)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                // Message to sign (usually a hash)
                let message_digest = b"Hello, ECDSA P-256!";
                
                // Sign the message digest
                match signing_key.sign_digest(message_digest) {
                    Ok(signature) => {
                        // Verify the signature
                        match verifying_key.verify_digest(message_digest, &signature) {
                            Ok(()) => {
                                // Signature verified successfully
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                        
                        // Test signature serialization
                        let signature_bytes = signature.to_bytes();
                        match EcdsaSignature::from_bytes(signature_bytes) {
                            Ok(recovered_sig) => {
                                // Signature successfully serialized and deserialized
                                let _ = recovered_sig.len();
                            },
                            Err(_) => {
                                // Serialization failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // ECDSA P-384 Example
    {
        let private_key = [0x03; 48]; // 384-bit private key
        let public_key = [0x04; 96];  // Uncompressed public key
        
        match (
            EcdsaSigningKey::new(&private_key, CurveType::P384),
            EcdsaVerifyingKey::new(&public_key, CurveType::P384)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message_digest = b"ECDSA P-384 signature test";
                
                match signing_key.sign_digest(message_digest) {
                    Ok(signature) => {
                        // P-384 signatures should be 96 bytes (48 bytes r + 48 bytes s)
                        assert!(signature.len() == 96);
                        
                        match verifying_key.verify_digest(message_digest, &signature) {
                            Ok(()) => {
                                // Success
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // ECDSA P-521 Example
    {
        let private_key = [0x05; 66]; // 521-bit private key (66 bytes)
        let public_key = [0x06; 132]; // Uncompressed public key
        
        match (
            EcdsaSigningKey::new(&private_key, CurveType::P521),
            EcdsaVerifyingKey::new(&public_key, CurveType::P521)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message_digest = b"ECDSA P-521 signature test with longer message";
                
                match signing_key.sign_digest(message_digest) {
                    Ok(signature) => {
                        // P-521 signatures should be 132 bytes (66 bytes r + 66 bytes s)
                        assert!(signature.len() == 132);
                        
                        match verifying_key.verify_digest(message_digest, &signature) {
                            Ok(()) => {
                                // Success
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }
}

/// Example using EdDSA signatures with Ed25519 and Ed448
fn eddsa_signature_example() {
    // Ed25519 Example
    {
        let private_key = [0x07; 64]; // Ed25519 private key (64 bytes: 32 secret + 32 public)
        let public_key = [0x08; 32];  // Ed25519 public key (32 bytes)
        
        // Create signing and verifying keys
        match (
            EddsaSigningKey::new(&private_key, EdwardsCurve::Ed25519),
            EddsaVerifyingKey::new(&public_key, EdwardsCurve::Ed25519)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                // Message to sign (EdDSA signs the full message, not a hash)
                let message = b"Hello, EdDSA Ed25519! This is a complete message.";
                
                // Sign the message
                match signing_key.sign_message(message) {
                    Ok(signature) => {
                        // Ed25519 signatures should be 64 bytes
                        assert!(signature.len() == 64);
                        
                        // Verify the signature
                        match verifying_key.verify_message(message, &signature) {
                            Ok(()) => {
                                // Signature verified successfully
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                        
                        // Test signature serialization
                        let signature_bytes = signature.to_bytes();
                        match EddsaSignature::from_bytes(signature_bytes) {
                            Ok(recovered_sig) => {
                                // Signature successfully serialized and deserialized
                                let _ = recovered_sig.len();
                            },
                            Err(_) => {
                                // Serialization failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // Ed448 Example
    {
        let private_key = [0x09; 114]; // Ed448 private key (114 bytes)
        let public_key = [0x0A; 57];   // Ed448 public key (57 bytes)
        
        match (
            EddsaSigningKey::new(&private_key, EdwardsCurve::Ed448),
            EddsaVerifyingKey::new(&public_key, EdwardsCurve::Ed448)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message = b"EdDSA Ed448 signature test with longer message for testing purposes";
                
                match signing_key.sign_message(message) {
                    Ok(signature) => {
                        // Ed448 signatures should be 114 bytes
                        assert!(signature.len() == 114);
                        
                        match verifying_key.verify_message(message, &signature) {
                            Ok(()) => {
                                // Success
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // Demonstrate EdDSA key features
    eddsa_key_features_example();
}

/// Example showing EdDSA-specific features
fn eddsa_key_features_example() {
    // EdDSA signs full messages (not pre-hashed)
    let private_key = [0x15; 64];
    let public_key = [0x16; 32];
    
    if let (Ok(signer), Ok(verifier)) = (
        EddsaSigningKey::new(&private_key, EdwardsCurve::Ed25519),
        EddsaVerifyingKey::new(&public_key, EdwardsCurve::Ed25519)
    ) {
        // EdDSA can sign messages of any length
        let short_message = b"Hi!";
        let long_message = b"This is a much longer message that demonstrates EdDSA's ability to sign arbitrary-length messages without requiring pre-hashing. Unlike ECDSA which typically signs message digests, EdDSA performs the hashing internally as part of the signature algorithm.";
        
        // Sign both short and long messages
        if let (Ok(short_sig), Ok(long_sig)) = (
            signer.sign_message(short_message),
            signer.sign_message(long_message)
        ) {
            // Both signatures should be the same length (64 bytes for Ed25519)
            assert!(short_sig.len() == 64);
            assert!(long_sig.len() == 64);
            
            // Verify both signatures
            if verifier.verify_message(short_message, &short_sig).is_ok() &&
               verifier.verify_message(long_message, &long_sig).is_ok() {
                // Both verifications succeeded
            }
        }
        
        // EdDSA is deterministic - same message produces same signature
        if let (Ok(sig1), Ok(sig2)) = (
            signer.sign_message(short_message),
            signer.sign_message(short_message)
        ) {
            // In real EdDSA, these should be identical (deterministic signatures)
            // Note: Our current implementation might not be fully deterministic due to 
            // placeholder elements, but real EdDSA would be
            let _ = (sig1.to_bytes(), sig2.to_bytes());
        }
    }
}

/// Example using RSA signatures with different key sizes
fn rsa_signature_example() {
    // RSA-2048 Example
    {
        let modulus = [0x10; 256];       // Mock RSA-2048 modulus (256 bytes)
        let private_exponent = [0x11; 256]; // Mock RSA-2048 private exponent
        let public_exponent = [0x01, 0x00, 0x01]; // 65537 as public exponent
        
        match (
            RsaSigningKey::new(&modulus, &private_exponent, RsaKeySize::Rsa2048),
            RsaVerifyingKey::new(&modulus, &public_exponent, RsaKeySize::Rsa2048)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message_digest = b"RSA-2048 signature test";
                
                match signing_key.sign_digest(message_digest, RsaHashAlgorithm::Sha256) {
                    Ok(signature) => {
                        // RSA-2048 signatures should be 256 bytes
                        assert!(signature.len() == 256);
                        
                        match verifying_key.verify_digest(message_digest, &signature, RsaHashAlgorithm::Sha256) {
                            Ok(()) => {
                                // Success
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                        
                        // Test signature serialization
                        let signature_bytes = signature.to_bytes();
                        match RsaSignature::from_bytes(signature_bytes) {
                            Ok(recovered_sig) => {
                                assert!(recovered_sig.len() == signature.len());
                            },
                            Err(_) => {
                                // Serialization failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // RSA-4096 Example
    {
        let modulus = [0x20; 512];       // Mock RSA-4096 modulus (512 bytes)
        let private_exponent = [0x21; 512]; // Mock RSA-4096 private exponent
        let public_exponent = [0x01, 0x00, 0x01]; // 65537 as public exponent
        
        match (
            RsaSigningKey::new(&modulus, &private_exponent, RsaKeySize::Rsa4096),
            RsaVerifyingKey::new(&modulus, &public_exponent, RsaKeySize::Rsa4096)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message_digest = b"RSA-4096 signature test with longer data";
                
                match signing_key.sign_digest(message_digest, RsaHashAlgorithm::Sha512) {
                    Ok(signature) => {
                        // RSA-4096 signatures should be 512 bytes
                        assert!(signature.len() == 512);
                        
                        match verifying_key.verify_digest(message_digest, &signature, RsaHashAlgorithm::Sha512) {
                            Ok(()) => {
                                // Success
                            },
                            Err(_) => {
                                // Verification failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }
}

/// Example using SM2 signatures (Chinese national standard)
fn sm2_signature_example() {
    // SM2 production curve example
    {
        let private_key = [0x30; 32]; // SM2 private key (32 bytes)
        let public_key = [0x31; 64];  // SM2 public key (64 bytes: x + y coordinates)
        let user_id = b"user@example.com"; // User ID for ZA computation
        
        match (
            Sm2SigningKey::new(&private_key, &public_key, Sm2Curve::Sm2, user_id),
            Sm2VerifyingKey::new(&public_key, Sm2Curve::Sm2)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message = b"SM2 signature test message";
                
                // Sign the message
                match signing_key.sign_message(message) {
                    Ok(signature) => {
                        // SM2 signatures should be 64 bytes
                        assert!(signature.len() == 64);
                        
                        // For verification, we need the same digest that was signed
                        // The signing key computes ZA internally, so we need to replicate this
                        if let Ok(za) = signing_key.compute_za() {
                            let mut digest = [0u8; 32];
                            for i in 0..32 {
                                digest[i] = za[i] ^ message.get(i % message.len()).copied().unwrap_or(0);
                            }
                            
                            match verifying_key.verify_digest(&digest, &signature) {
                                Ok(()) => {
                                    // Success
                                },
                                Err(_) => {
                                    // Verification failed
                                }
                            }
                        }
                        
                        // Test signature serialization
                        let signature_bytes = signature.to_bytes();
                        match Sm2Signature::from_bytes(signature_bytes) {
                            Ok(recovered_sig) => {
                                assert!(recovered_sig.len() == signature.len());
                            },
                            Err(_) => {
                                // Serialization failed
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // SM2 test curve example
    {
        let private_key = [0x32; 32]; 
        let public_key = [0x33; 64];  
        let user_id = b"test@example.com";
        
        match (
            Sm2SigningKey::new(&private_key, &public_key, Sm2Curve::Sm2Test, user_id),
            Sm2VerifyingKey::new(&public_key, Sm2Curve::Sm2Test)
        ) {
            (Ok(signing_key), Ok(verifying_key)) => {
                let message = b"SM2 test curve signature";
                
                match signing_key.sign_message(message) {
                    Ok(signature) => {
                        // Test ZA computation
                        if let Ok(za) = signing_key.compute_za() {
                            // ZA should be 32 bytes (SHA-256 output)
                            assert!(za.len() == 32);
                            
                            // Compute the digest that was actually signed
                            let mut digest = [0u8; 32];
                            for i in 0..32 {
                                digest[i] = za[i] ^ message.get(i % message.len()).copied().unwrap_or(0);
                            }
                            
                            match verifying_key.verify_digest(&digest, &signature) {
                                Ok(()) => {
                                    // Test curve verification succeeded
                                },
                                Err(_) => {
                                    // Verification failed
                                }
                            }
                        }
                    },
                    Err(_) => {
                        // Signing failed
                    }
                }
            },
            _ => {
                // Key creation failed
            }
        }
    }

    // Test different user IDs
    sm2_user_id_examples();
}

/// Example showing SM2 user ID features
fn sm2_user_id_examples() {
    let private_key = [0x34; 32];
    let public_key = [0x35; 64];
    
    // Different user IDs produce different ZA values
    let user_ids = [
        b"alice@company.com".as_slice(),
        b"bob@company.com".as_slice(),
        b"charlie@company.com".as_slice(),
    ];
    
    for user_id in user_ids.iter() {
        if let Ok(signing_key) = Sm2SigningKey::new(&private_key, &public_key, Sm2Curve::Sm2, user_id) {
            // Each user ID should produce a different ZA value
            if let Ok(za) = signing_key.compute_za() {
                // ZA incorporates the user ID, so different user IDs = different ZA
                assert!(za.len() == 32);
                
                // Sign a message
                let message = b"Common message for all users";
                if let Ok(signature) = signing_key.sign_message(message) {
                    // Each user will produce a different signature for the same message
                    // because ZA is different
                    assert!(signature.len() == 64);
                }
            }
        }
    }
}

/// Example showing signature error handling patterns
fn signature_error_handling_example() {
    // Test invalid key sizes for ECDSA
    {
        let wrong_private_key = [0x01; 31]; // Wrong size for P-256 (should be 32)
        let wrong_public_key = [0x02; 63];  // Wrong size for P-256 (should be 64)
        
        match EcdsaSigningKey::new(&wrong_private_key, CurveType::P256) {
            Ok(_) => {
                // This should not happen with wrong key size
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for wrong key size
            },
            Err(_) => {
                // Other error
            }
        }
        
        match EcdsaVerifyingKey::new(&wrong_public_key, CurveType::P256) {
            Ok(_) => {
                // This should not happen with wrong key size
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for wrong key size
            },
            Err(_) => {
                // Other error
            }
        }
    }

    // Test signature length validation
    {
        let _valid_key = [0x01; 32];
        if let Ok(verifying_key) = EcdsaVerifyingKey::new(&[0x02; 64], CurveType::P256) {
            // Create signature with wrong length
            let wrong_signature_bytes = [0xff; 32]; // Too short for P-256 (should be 64)
            
            if let Ok(wrong_signature) = EcdsaSignature::from_bytes(&wrong_signature_bytes) {
                match verifying_key.verify_digest(b"test", &wrong_signature) {
                    Ok(()) => {
                        // This might happen in placeholder implementation
                    },
                    Err(CmoxError::AuthenticationFailed) => {
                        // Expected for wrong signature length
                    },
                    Err(_) => {
                        // Other error
                    }
                }
            }
        }
    }

    // Test signature serialization limits
    {
        let too_large_signature = [0xff; 200]; // Larger than max ECDSA signature
        match EcdsaSignature::from_bytes(&too_large_signature) {
            Ok(_) => {
                // This should not happen
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for oversized signature
            },
            Err(_) => {
                // Other error
            }
        }
        
        let too_large_rsa_signature = [0xff; 600]; // Larger than max RSA signature
        match RsaSignature::from_bytes(&too_large_rsa_signature) {
            Ok(_) => {
                // This should not happen  
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for oversized signature
            },
            Err(_) => {
                // Other error
            }
        }
    }

    // Test SM2 error handling
    {
        let wrong_private_key = [0x01; 31]; // Wrong size for SM2 (should be 32)
        let wrong_public_key = [0x02; 63];  // Wrong size for SM2 (should be 64)
        let user_id = b"test@example.com";
        
        match Sm2SigningKey::new(&wrong_private_key, &wrong_public_key, Sm2Curve::Sm2, user_id) {
            Ok(_) => {
                // This should not happen with wrong key sizes
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for wrong key sizes
            },
            Err(_) => {
                // Other error
            }
        }
        
        // Test user ID too long
        let too_long_user_id = [0x42; 65]; // Max user ID is 64 bytes
        match Sm2SigningKey::new(&[0x01; 32], &[0x02; 64], Sm2Curve::Sm2, &too_long_user_id) {
            Ok(_) => {
                // This should not happen with oversized user ID
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for oversized user ID
            },
            Err(_) => {
                // Other error
            }
        }
        
        // Test SM2 signature serialization
        let wrong_sm2_signature = [0xff; 63]; // Wrong size for SM2 (should be 64)
        match Sm2Signature::from_bytes(&wrong_sm2_signature) {
            Ok(_) => {
                // This should not happen
            },
            Err(CmoxError::InvalidInput) => {
                // Expected error for wrong signature size
            },
            Err(_) => {
                // Other error
            }
        }
    }
}

/// Example showing different signature use patterns
fn signature_usage_patterns() {
    // Pattern 1: Sign and verify with same keys
    let private_key = [0x42; 32];
    let public_key = [0x43; 64];
    
    if let (Ok(signer), Ok(verifier)) = (
        EcdsaSigningKey::new(&private_key, CurveType::P256),
        EcdsaVerifyingKey::new(&public_key, CurveType::P256)
    ) {
        let message = b"Important document to sign";
        
        if let Ok(signature) = signer.sign_digest(message) {
            // Signature can be stored or transmitted
            let signature_bytes = signature.to_bytes();
            
            // Later, reconstruct signature and verify
            if let Ok(reconstructed_sig) = EcdsaSignature::from_bytes(signature_bytes) {
                if verifier.verify_digest(message, &reconstructed_sig).is_ok() {
                    // Document authenticity confirmed
                }
            }
        }
    }
    
    // Pattern 2: Batch verification of multiple signatures
    let messages = [
        b"Message 1".as_slice(),
        b"Message 2".as_slice(), 
        b"Message 3".as_slice(),
    ];
    
    if let Ok(signer) = EcdsaSigningKey::new(&private_key, CurveType::P256) {
        let mut signatures = [const { None }; 3];
        
        // Sign all messages
        for (i, message) in messages.iter().enumerate() {
            if let Ok(sig) = signer.sign_digest(message) {
                signatures[i] = Some(sig);
            }
        }
        
        // Verify all signatures
        if let Ok(verifier) = EcdsaVerifyingKey::new(&public_key, CurveType::P256) {
            for (i, signature) in signatures.iter().enumerate() {
                if let Some(sig) = signature {
                    if verifier.verify_digest(messages[i], sig).is_ok() {
                        // Message i verified successfully
                    }
                }
            }
        }
    }
}