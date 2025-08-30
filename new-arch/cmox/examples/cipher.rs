//! Comprehensive Cipher and AEAD Example
//!
//! This example demonstrates all available cipher modes and AEAD algorithms
//! in the CMOX crate, showing both basic usage and advanced features.

#![no_std]
#![no_main]

use aead::{AeadInPlace, Key, KeyInit, Nonce};
use cmox::aead::{Aes128FastCcm, Aes128FastGcmFast, ChaChaPoly};
use cmox::initialize;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    let key = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ];
    let nonce = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
    ];
    let aad = b"Additional authenticated data";
    let plaintext = b"Secret message for AEAD encryption";

    // AES-GCM example
    aes_gcm_example(&key, &nonce, aad, plaintext);

    // AES-CCM example
    aes_ccm_example(&key, &nonce, aad, plaintext);

    // ChaCha20-Poly1305 example
    chacha20_poly1305_example(&nonce, aad, plaintext);

    loop {}
}

/// AES-GCM AEAD example
fn aes_gcm_example(key: &[u8], nonce: &[u8], aad: &[u8], plaintext: &[u8]) {
    let key_ref = Key::<Aes128FastCcm>::from_slice(key);
    let cipher = Aes128FastGcmFast::new(key_ref);

    let mut buffer = [0u8; 64];
    let len = plaintext.len().min(48);
    buffer[..len].copy_from_slice(&plaintext[..len]);

    // Encrypt and authenticate
    let nonce_ref = Nonce::<Aes128FastGcmFast>::from_slice(nonce);
    let tag = cipher
        .encrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len])
        .expect("GCM encryption failed");

    // Decrypt and verify
    cipher
        .decrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len], &tag)
        .expect("GCM decryption failed");

    // Verify decrypted matches original plaintext
    assert_eq!(plaintext, &buffer[..len]);
}

/// AES-CCM AEAD example
fn aes_ccm_example(key: &[u8], nonce: &[u8], aad: &[u8], plaintext: &[u8]) {
    let key_ref = Key::<Aes128FastCcm>::from_slice(key);
    let cipher = Aes128FastCcm::new(key_ref);

    let mut buffer = [0u8; 64];
    let len = plaintext.len().min(48);
    buffer[..len].copy_from_slice(&plaintext[..len]);

    // Encrypt and authenticate
    let nonce_ref = Nonce::<Aes128FastCcm>::from_slice(nonce);
    let tag = cipher
        .encrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len])
        .expect("CCM encryption failed");

    // Decrypt and verify
    cipher
        .decrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len], &tag)
        .expect("CCM decryption failed");

    // Verify decrypted matches original plaintext
    assert_eq!(plaintext, &buffer[..len]);
}

/// ChaCha20-Poly1305 AEAD example
fn chacha20_poly1305_example(nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) {
    let key = [
        0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e,
        0x8f, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d,
        0x9e, 0x9f,
    ];

    let key_ref = Key::<ChaChaPoly>::from_slice(&key);
    let cipher = ChaChaPoly::new(key_ref);

    let mut buffer = [0u8; 64];
    let len = plaintext.len().min(48);
    buffer[..len].copy_from_slice(&plaintext[..len]);

    // Encrypt and authenticate
    let nonce_ref = Nonce::<ChaChaPoly>::from_slice(nonce);
    let tag = cipher
        .encrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len])
        .expect("ChaCha20-Poly1305 encryption failed");

    // Decrypt and verify
    cipher
        .decrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len], &tag)
        .expect("ChaCha20-Poly1305 decryption failed");

    // Verify decrypted matches original plaintext
    assert_eq!(plaintext, &buffer[..len]);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
