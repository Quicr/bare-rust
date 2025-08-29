//! Example demonstrating block cipher usage with the CMOX crate
//!
//! This example shows how to use AES and SM4 block ciphers for encryption and decryption.

#![no_std]
#![no_main]

use cipher::{Block, KeyInit};
use cmox::{
    cipher::{Aes128, Aes192, Aes256, Sm4},
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

    // AES-128 ECB Example
    {
        let key = b"1234567890123456"; // 16-byte key for AES-128
        let plaintext = b"Hello, World!123"; // Must be exactly 16 bytes for ECB

        // Create AES-128 cipher using cipher crate KeyInit trait
        let cipher = Aes128::new_from_slice(key).expect("Invalid key length");

        // Convert plaintext to block format
        let mut block = Block::<Aes128>::clone_from_slice(plaintext);

        // Encrypt the block in-place
        cipher
            .encrypt_block_inplace(&mut block)
            .expect("Encryption failed");

        // block now contains encrypted data
        let encrypted = block;

        // Decrypt the block
        let mut decrypted_block = encrypted;
        cipher
            .decrypt_block_inplace(&mut decrypted_block)
            .expect("Decryption failed");

        // Verify decryption worked
        if decrypted_block.as_slice() == plaintext {
            // Encryption/decryption successful
        }
    }

    // AES-256 ECB Example using native API
    {
        let key = b"12345678901234567890123456789012"; // 32-byte key for AES-256
        let plaintext = b"Native API usage"; // Pad to 16 bytes

        // Create AES-256 cipher using native API
        let cipher = Aes256::new_with_key(key).expect("Failed to create AES-256");

        // Convert to block and pad if necessary
        let mut padded_plaintext = [0u8; 16];
        let len = core::cmp::min(plaintext.len(), 16);
        padded_plaintext[..len].copy_from_slice(&plaintext[..len]);

        let block = Block::<Aes256>::clone_from_slice(&padded_plaintext);

        // Encrypt using the native encrypt_block method
        let encrypted_block = cipher.encrypt_block(&block).expect("Encryption failed");

        // Decrypt back to verify
        let decrypted_block = cipher
            .decrypt_block(&encrypted_block)
            .expect("Decryption failed");

        // Compare original and decrypted
        if decrypted_block == block {
            // Success!
        }
    }

    // SM4 ECB Example
    {
        let key = b"sm4key1234567890"; // 16-byte key for SM4
        let plaintext = b"SM4 cipher test!"; // Exactly 16 bytes

        // Create SM4 cipher
        let cipher = Sm4::new_with_key(key).expect("Failed to create SM4");

        let mut block = Block::<Sm4>::clone_from_slice(plaintext);

        // Encrypt
        cipher
            .encrypt_block_inplace(&mut block)
            .expect("SM4 encryption failed");
        let encrypted = block;

        // Decrypt
        let mut decrypted = encrypted;
        cipher
            .decrypt_block_inplace(&mut decrypted)
            .expect("SM4 decryption failed");

        if decrypted.as_slice() == plaintext {
            // SM4 encryption/decryption successful
        }
    }

    // Multiple block encryption example (simplified CBC-like mode)
    {
        let key = b"1234567890123456";
        let cipher = Aes128::new_with_key(key).expect("Failed to create AES-128");

        // Simulate multiple blocks of data
        let blocks = [
            b"Block 1 data!!!!",
            b"Block 2 data!!!!",
            b"Block 3 data!!!!",
        ];

        let mut encrypted_blocks = [[0u8; 16]; 3];

        // Encrypt each block
        for (i, plaintext_block) in blocks.iter().enumerate() {
            let mut block = Block::<Aes128>::clone_from_slice(plaintext_block.as_slice());
            cipher
                .encrypt_block_inplace(&mut block)
                .expect("Multi-block encryption failed");
            encrypted_blocks[i] = *block.as_ref();
        }

        // Decrypt each block back
        for (i, encrypted_block) in encrypted_blocks.iter().enumerate() {
            let mut block = Block::<Aes128>::clone_from_slice(encrypted_block);
            cipher
                .decrypt_block_inplace(&mut block)
                .expect("Multi-block decryption failed");

            if block.as_slice() == blocks[i] {
                // Block encryption/decryption successful
            }
        }
    }

    // Key size demonstration
    {
        // AES supports different key sizes
        let aes128_key = b"1234567890123456"; // 16 bytes
        let aes192_key = b"123456789012345678901234"; // 24 bytes
        let aes256_key = b"12345678901234567890123456789012"; // 32 bytes

        let _cipher128 = Aes128::new_with_key(aes128_key).expect("AES-128 key");
        let _cipher192 = Aes192::new_with_key(aes192_key).expect("AES-192 key");
        let _cipher256 = Aes256::new_with_key(aes256_key).expect("AES-256 key");

        // SM4 uses 16-byte keys
        let sm4_key = b"sm4key1234567890";
        let _sm4_cipher = Sm4::new_with_key(sm4_key).expect("SM4 key");
    }

    // Error handling demonstration
    {
        // Trying to use wrong key size will fail
        let wrong_key = b"wrong_size";

        // This would panic in new(), but new_with_key() returns Result
        match Aes128::new_with_key(wrong_key) {
            Ok(_) => {
                // This shouldn't happen with wrong key size
            }
            Err(_) => {
                // Expected - key size mismatch
            }
        }
    }
}
