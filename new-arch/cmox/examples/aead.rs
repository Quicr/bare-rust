//! AEAD (Authenticated Encryption with Associated Data) examples using CMOX library
//!
//! This example demonstrates the usage of AES-GCM AEAD ciphers with the CMOX library,
//! including both native API usage and standard Rust Crypto trait compatibility.

#![no_std]
#![no_main]

use cmox::{
    aead::{Aes128FastGcmFast, Aes256FastGcmFast},
    initialize,
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // Examples using standard Rust Crypto AEAD traits
    aes_gcm_trait_example();
}

/// Example using standard AEAD traits (AeadInPlace)
fn aes_gcm_trait_example() {
    use aead::{AeadInPlace, KeyInit};

    // AES-128-GCM trait-based example
    {
        let key = aead::Key::<Aes128FastGcmFast>::from_slice(&[0x42; 16]);
        let cipher = Aes128FastGcmFast::new(&key);

        let nonce = aead::Nonce::<Aes128FastGcmFast>::from_slice(&[0x01; 12]);
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
        let key = aead::Key::<Aes256FastGcmFast>::from_slice(&[0x84; 32]);
        let cipher = Aes256FastGcmFast::new(key);

        let nonce_bytes = [0x02; 12];
        let nonce = aead::Nonce::<Aes256FastGcmFast>::from_slice(&nonce_bytes);
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
