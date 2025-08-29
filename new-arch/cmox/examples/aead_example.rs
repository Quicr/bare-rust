//! AEAD (Authenticated Encryption with Associated Data) examples using CMOX library
//!
//! This example demonstrates the usage of AES-GCM AEAD ciphers with the CMOX library,
//! including both native API usage and standard Rust Crypto trait compatibility.

#![no_std]
#![no_main]

use cmox::{
    aead::{Aes128Gcm, Aes256Gcm},
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

    // Run AEAD examples
    aes_gcm_native_api_example();
    aead_usage_patterns();
    error_handling_example();

    // Examples using standard Rust Crypto AEAD traits
    aes_gcm_trait_example();
}

/// Example using native AES-GCM API
fn aes_gcm_native_api_example() {
    // AES-128-GCM Example
    {
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f,
        ];

        let nonce = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        ];

        let associated_data = b"additional authenticated data";

        let plaintext = *b"Hello, AES-GCM!"; // 16 bytes
        let mut ciphertext = plaintext;

        // Create AES-128-GCM cipher
        match Aes128Gcm::new_with_key(&key) {
            Ok(mut cipher) => {
                // Encrypt the data
                match cipher.encrypt_inplace(&nonce, associated_data, &mut ciphertext) {
                    Ok(tag) => {
                        // Ciphertext is now encrypted using real CMOX operations
                        // and tag contains the authentication tag from CMOX GCM

                        // Decrypt the data
                        let mut decrypted = ciphertext;
                        match cipher.decrypt_inplace(&nonce, associated_data, &mut decrypted, &tag)
                        {
                            Ok(()) => {
                                // Verify decryption succeeded
                                // In real implementation, would check decrypted == plaintext
                            }
                            Err(_) => {
                                // Handle decryption error
                            }
                        }
                    }
                    Err(_) => {
                        // Handle encryption error
                    }
                }
            }
            Err(_) => {
                // Handle cipher creation error
            }
        }
    }

    // AES-256-GCM Example
    {
        let key = [0x00; 32]; // 256-bit key
        let nonce = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        ];
        let associated_data = b"AES-256-GCM test data";

        let mut data = *b"This is a longer message for AES-256-GCM testing."; // 50 bytes

        match Aes256Gcm::new_with_key(&key) {
            Ok(mut cipher) => {
                match cipher.encrypt_inplace(&nonce, associated_data, &mut data) {
                    Ok(tag) => {
                        // Store the tag for later verification
                        let auth_tag = tag;

                        // For demonstration, decrypt the same data
                        match cipher.decrypt_inplace(&nonce, associated_data, &mut data, &auth_tag)
                        {
                            Ok(()) => {
                                // Success - data should be back to original plaintext
                            }
                            Err(_) => {
                                // Authentication failed or other error
                            }
                        }
                    }
                    Err(_) => {
                        // Encryption failed
                    }
                }
            }
            Err(_) => {
                // Cipher initialization failed
            }
        }
    }
}

/// Example showing proper AEAD usage patterns
fn aead_usage_patterns() {
    let key = [0x42; 16]; // AES-128 key
    let nonce = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
    ];

    // Pattern 1: Encrypt with associated data
    {
        let associated_data = b"message metadata";
        let mut message = *b"secret message";

        if let Ok(mut cipher) = Aes128Gcm::new_with_key(&key) {
            if let Ok(tag) = cipher.encrypt_inplace(&nonce, associated_data, &mut message) {
                // message now contains ciphertext
                // tag contains authentication tag
                // Both are needed for decryption

                // To decrypt:
                if let Ok(()) = cipher.decrypt_inplace(&nonce, associated_data, &mut message, &tag)
                {
                    // message now contains original plaintext
                }
            }
        }
    }

    // Pattern 2: Encrypt without associated data
    {
        let mut message = *b"another secret";

        if let Ok(mut cipher) = Aes128Gcm::new_with_key(&key) {
            // Empty associated data
            if let Ok(tag) = cipher.encrypt_inplace(&nonce, &[], &mut message) {
                // Decrypt without associated data
                if let Ok(()) = cipher.decrypt_inplace(&nonce, &[], &mut message, &tag) {
                    // Success
                }
            }
        }
    }
}

/// Example showing error handling patterns
fn error_handling_example() {
    let key = [0x00; 16];
    let nonce = [0x00; 12];
    let mut data = [0x00; 16];
    let _tag = [0x00; 16];

    match Aes128Gcm::new_with_key(&key) {
        Ok(mut cipher) => {
            // Test decryption with wrong tag (should fail)
            let wrong_tag = [0xff; 16];
            match cipher.decrypt_inplace(&nonce, &[], &mut data, &wrong_tag) {
                Ok(()) => {
                    // This should not happen with a wrong tag in real implementation
                }
                Err(CmoxError::AuthenticationFailed) => {
                    // Expected error for wrong tag
                }
                Err(_) => {
                    // Other error
                }
            }
        }
        Err(CmoxError::InitializationFailed) => {
            // Handle cipher creation failure
        }
        Err(_) => {
            // Handle other errors
        }
    }
}

/// Example using standard AEAD traits (AeadInPlace)
fn aes_gcm_trait_example() {
    use aead::{AeadInPlace, KeyInit};

    // AES-128-GCM trait-based example
    {
        let key = [0x42; 16];
        let cipher = Aes128Gcm::new(&key.into());

        let nonce_bytes = [0x01; 12];
        let nonce = aead::Nonce::<Aes128Gcm>::from_slice(&nonce_bytes);
        let aad = b"trait-based associated data";

        // Test in-place encryption
        let mut buffer = *b"Hello, AEAD trait!"; // Must be exact size for in-place
        let _original = buffer; // Store original for potential verification

        match cipher.encrypt_in_place_detached(nonce, aad, &mut buffer) {
            Ok(tag) => {
                // Buffer now contains ciphertext
                // Tag contains authentication tag

                // Decrypt back to verify
                match cipher.decrypt_in_place_detached(nonce, aad, &mut buffer, &tag) {
                    Ok(()) => {
                        // Buffer should now contain original plaintext
                        // In practice, verify buffer == original
                    }
                    Err(_) => {
                        // Decryption or authentication failed
                    }
                }
            }
            Err(_) => {
                // Encryption failed
            }
        }
    }

    // AES-256-GCM trait-based example
    {
        let key = [0x84; 32]; // 256-bit key
        let cipher = Aes256Gcm::new(&key.into());

        let nonce_bytes = [0x02; 12];
        let nonce = aead::Nonce::<Aes256Gcm>::from_slice(&nonce_bytes);
        let aad = b"AES-256 trait example";

        let mut buffer = *b"AES-256-GCM with traits works great!";

        match cipher.encrypt_in_place_detached(nonce, aad, &mut buffer) {
            Ok(tag) => {
                // Test that we can decrypt successfully
                match cipher.decrypt_in_place_detached(nonce, aad, &mut buffer, &tag) {
                    Ok(()) => {
                        // Success - demonstrates trait compatibility
                    }
                    Err(_) => {
                        // Authentication failed
                    }
                }
            }
            Err(_) => {
                // Encryption failed
            }
        }
    }
}
