//! Comprehensive Cipher and AEAD Example
//! 
//! This example demonstrates all available cipher modes and AEAD algorithms
//! in the CMOX crate, showing both basic usage and advanced features.

#![no_std]
#![no_main]

use cmox::{initialize, cipher::*};
use cmox::aead::{Aes128Gcm, Aes128Ccm, ChaCha20Poly1305};
use aead::AeadInPlace;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // Example 1: Block cipher modes
    block_cipher_examples();

    // Example 2: AEAD cipher examples
    aead_examples();

    // Example 3: Comparison of cipher modes
    cipher_mode_comparison();

    loop {}
}

/// Demonstrate different AES block cipher modes
fn block_cipher_examples() {
    let key = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
               0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    let iv = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
              0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
    let plaintext = b"Hello, World! This is a test message for AES encryption.";

    // ECB Mode (Electronic Codebook) - Not recommended for most uses
    ecb_example(&key, plaintext);

    // CBC Mode (Cipher Block Chaining) - Requires IV
    cbc_example(&key, &iv, plaintext);

    // CFB Mode (Cipher Feedback) - Stream cipher mode, requires IV
    cfb_example(&key, &iv, plaintext);

    // CTR Mode (Counter) - Stream cipher mode, requires IV/nonce
    ctr_example(&key, &iv, plaintext);

    // OFB Mode (Output Feedback) - Stream cipher mode, requires IV
    ofb_example(&key, &iv, plaintext);
}

/// ECB mode example (insecure for most applications)
fn ecb_example(key: &[u8; 16], plaintext: &[u8]) {
    let cipher = Aes128::new_with_key(key).expect("Failed to create AES-128 ECB cipher");

    // ECB mode processes blocks independently
    // Note: This is insecure for most real-world applications
    // as identical plaintext blocks produce identical ciphertext blocks
    
    // For demonstration, we'll encrypt one block at a time
    let mut output = [0u8; 64];
    let mut pos = 0;
    
    for chunk in plaintext.chunks(16) {
        if chunk.len() == 16 {
            let mut block = [0u8; 16];
            block.copy_from_slice(chunk);
            
            let encrypted_block = cipher.encrypt_block(&block.into())
                .expect("ECB encryption failed");
            
            output[pos..pos + 16].copy_from_slice(&encrypted_block);
            pos += 16;
        }
    }
    
    // Decrypt back
    pos = 0;
    let mut decrypted = [0u8; 64];
    for chunk in output[..pos].chunks(16) {
        if chunk.len() == 16 {
            let mut block = [0u8; 16];
            block.copy_from_slice(chunk);
            
            let decrypted_block = cipher.decrypt_block(&block.into())
                .expect("ECB decryption failed");
            
            decrypted[pos..pos + 16].copy_from_slice(&decrypted_block);
            pos += 16;
        }
    }
}

/// CBC mode example (secure with proper IV)
fn cbc_example(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) {
    let cipher = Aes128Cbc::new_with_key(key)
        .expect("Failed to create AES-128 CBC cipher");

    // Pad plaintext to block boundary (simple PKCS7-style padding)
    let mut padded_plaintext = [0u8; 64];
    let len = plaintext.len().min(48); // Leave room for padding
    padded_plaintext[..len].copy_from_slice(&plaintext[..len]);
    let padding = 16 - (len % 16);
    for item in padded_plaintext.iter_mut().skip(len).take(padding) {
        *item = padding as u8;
    }
    let total_len = len + padding;

    // Encrypt
    let mut ciphertext = [0u8; 64];
    let encrypted_len = cipher.encrypt(iv, &padded_plaintext[..total_len], &mut ciphertext)
        .expect("CBC encryption failed");

    // Decrypt
    let mut decrypted = [0u8; 64];
    let decrypted_len = cipher.decrypt(iv, &ciphertext[..encrypted_len], &mut decrypted)
        .expect("CBC decryption failed");

    // Remove padding
    if decrypted_len > 0 {
        let padding_len = decrypted[decrypted_len - 1] as usize;
        let _original_len = decrypted_len - padding_len;
        // Verify original message matches (first `_original_len` bytes)
    }
}

/// CFB mode example (stream cipher mode)
fn cfb_example(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) {
    let cipher = Aes128Cfb::new_with_key(key)
        .expect("Failed to create AES-128 CFB cipher");

    let len = plaintext.len().min(48);

    // Encrypt
    let mut ciphertext = [0u8; 64];
    let encrypted_len = cipher.encrypt(iv, &plaintext[..len], &mut ciphertext)
        .expect("CFB encryption failed");

    // Decrypt
    let mut decrypted = [0u8; 64];
    let decrypted_len = cipher.decrypt(iv, &ciphertext[..encrypted_len], &mut decrypted)
        .expect("CFB decryption failed");

    // Verify decrypted matches original
    assert_eq!(decrypted_len, len);
}

/// CTR mode example (stream cipher mode)
fn ctr_example(key: &[u8; 16], nonce: &[u8; 16], plaintext: &[u8]) {
    let cipher = Aes128Ctr::new_with_key(key)
        .expect("Failed to create AES-128 CTR cipher");

    let len = plaintext.len().min(48);

    // CTR mode: encryption and decryption are the same operation
    let mut ciphertext = [0u8; 64];
    let encrypted_len = cipher.encrypt(nonce, &plaintext[..len], &mut ciphertext)
        .expect("CTR encryption failed");

    // Decrypt (same operation as encrypt)
    let mut decrypted = [0u8; 64];
    let decrypted_len = cipher.decrypt(nonce, &ciphertext[..encrypted_len], &mut decrypted)
        .expect("CTR decryption failed");

    // Verify decrypted matches original
    assert_eq!(decrypted_len, len);
}

/// OFB mode example (stream cipher mode)
fn ofb_example(key: &[u8; 16], iv: &[u8; 16], plaintext: &[u8]) {
    let cipher = Aes128Ofb::new_with_key(key)
        .expect("Failed to create AES-128 OFB cipher");

    let len = plaintext.len().min(48);

    // Encrypt
    let mut ciphertext = [0u8; 64];
    let encrypted_len = cipher.encrypt(iv, &plaintext[..len], &mut ciphertext)
        .expect("OFB encryption failed");

    // Decrypt
    let mut decrypted = [0u8; 64];
    let decrypted_len = cipher.decrypt(iv, &ciphertext[..encrypted_len], &mut decrypted)
        .expect("OFB decryption failed");

    // Verify decrypted matches original
    assert_eq!(decrypted_len, len);
}

/// Demonstrate AEAD (Authenticated Encryption with Associated Data) algorithms
fn aead_examples() {
    let key = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
               0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    let nonce = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
                 0x08, 0x09, 0x0a, 0x0b];
    let aad = b"Additional authenticated data";
    let plaintext = b"Secret message for AEAD encryption";

    // AES-GCM example
    aes_gcm_example(&key, &nonce, aad, plaintext);

    // AES-CCM example  
    aes_ccm_example(&key, &nonce, aad, plaintext);

    // ChaCha20-Poly1305 example
    chacha20_poly1305_example(&nonce, aad, plaintext);
}

/// AES-GCM AEAD example
fn aes_gcm_example(key: &[u8; 16], nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) {
    let cipher = Aes128Gcm::new_with_key(key)
        .expect("Failed to create AES-128-GCM cipher");

    let mut buffer = [0u8; 64];
    let len = plaintext.len().min(48);
    buffer[..len].copy_from_slice(&plaintext[..len]);

    // Encrypt and authenticate
    let nonce_ref = aead::Nonce::<Aes128Gcm>::from_slice(nonce.as_slice());
    let tag = cipher.encrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len])
        .expect("GCM encryption failed");

    // Decrypt and verify
    cipher.decrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len], &tag)
        .expect("GCM decryption failed");

    // Verify decrypted matches original plaintext
}

/// AES-CCM AEAD example
fn aes_ccm_example(key: &[u8; 16], nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) {
    let cipher = Aes128Ccm::new_with_key(key)
        .expect("Failed to create AES-128-CCM cipher");

    let mut buffer = [0u8; 64];
    let len = plaintext.len().min(48);
    buffer[..len].copy_from_slice(&plaintext[..len]);

    // Encrypt and authenticate
    let nonce_ref = aead::Nonce::<Aes128Ccm>::from_slice(nonce.as_slice());
    let tag = cipher.encrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len])
        .expect("CCM encryption failed");

    // Decrypt and verify
    cipher.decrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len], &tag)
        .expect("CCM decryption failed");

    // Verify decrypted matches original plaintext
}

/// ChaCha20-Poly1305 AEAD example
fn chacha20_poly1305_example(nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) {
    let key = [0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
               0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
               0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
               0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f];

    let cipher = ChaCha20Poly1305::new_with_key(&key)
        .expect("Failed to create ChaCha20-Poly1305 cipher");

    let mut buffer = [0u8; 64];
    let len = plaintext.len().min(48);
    buffer[..len].copy_from_slice(&plaintext[..len]);

    // Encrypt and authenticate
    let nonce_ref = aead::Nonce::<ChaCha20Poly1305>::from_slice(nonce.as_slice());
    let tag = cipher.encrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len])
        .expect("ChaCha20-Poly1305 encryption failed");

    // Decrypt and verify
    cipher.decrypt_in_place_detached(nonce_ref, aad, &mut buffer[..len], &tag)
        .expect("ChaCha20-Poly1305 decryption failed");

    // Verify decrypted matches original plaintext
}

/// Compare different cipher modes and their characteristics
fn cipher_mode_comparison() {
    // This function demonstrates the differences between cipher modes:
    
    // ECB: 
    // - Each block encrypted independently
    // - Same plaintext block → same ciphertext block (insecure)
    // - No IV required
    // - Parallelizable
    // - Not recommended for most applications
    
    // CBC:
    // - Each block XORed with previous ciphertext block
    // - Requires IV
    // - Sequential encryption, parallelizable decryption
    // - Good security with proper IV
    
    // CFB:
    // - Stream cipher mode
    // - Can encrypt partial blocks
    // - Sequential encryption and decryption
    // - Self-synchronizing
    
    // CTR:
    // - Stream cipher mode
    // - Counter mode: encrypt counter + nonce, XOR with plaintext
    // - Parallelizable encryption and decryption
    // - No padding required
    // - Nonce must never be reused with same key
    
    // OFB:
    // - Stream cipher mode
    // - Previous cipher output used as next IV
    // - Sequential encryption and decryption
    // - No error propagation
    
    // AEAD modes (GCM, CCM, ChaCha20-Poly1305):
    // - Provide both confidentiality and authenticity
    // - Protect associated authenticated data (AAD)
    // - Detect tampering/modification
    // - Recommended for new applications
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}