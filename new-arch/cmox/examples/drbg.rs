//! CTR-DRBG Random Number Generator Example
//!
//! This example demonstrates how to use the CTR-DRBG implementation
//! to generate cryptographically secure random numbers.

#![no_std]
#![no_main]

use cmox::{
    drbg::{CtrDrbg, CtrDrbgVariant},
    initialize,
};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn main() -> ! {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // Example 1: Basic CTR-DRBG usage with AES-256
    basic_ctr_drbg_example();

    // Example 2: Different algorithm variants
    algorithm_variants_example();

    // Example 3: Reseeding example
    reseeding_example();

    // Example 4: Convenience methods
    convenience_methods_example();

    loop {}
}

/// Basic CTR-DRBG example with AES-256
fn basic_ctr_drbg_example() {
    // High-quality entropy (in practice, this should come from a true random source)
    let entropy = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e,
        0x1f, 0x20,
    ];

    // Nonce for additional randomness
    let nonce = [
        0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
        0xb0,
    ];

    // Optional personalization string for domain separation
    let personalization = b"Example Application RNG";

    // Create CTR-DRBG with AES-256
    let mut rng = CtrDrbg::new(
        CtrDrbgVariant::Aes256Fast,
        &entropy,
        &nonce,
        Some(personalization),
    )
    .expect("Failed to create CTR-DRBG");

    // Generate 32 bytes of random data
    let mut random_bytes = [0u8; 32];
    rng.generate_bytes(&mut random_bytes)
        .expect("Failed to generate random bytes");

    // Generate random data with additional input
    let additional_input = b"session_key_generation";
    let mut session_key = [0u8; 16];
    rng.generate(&mut session_key, Some(additional_input))
        .expect("Failed to generate session key");
}

/// Example showing different algorithm variants
fn algorithm_variants_example() {
    let entropy_128 = [0x42u8; 16]; // Minimum for AES-128
    let entropy_256 = [0x42u8; 32]; // Minimum for AES-256
    let nonce = [0x43u8; 16];

    // AES-128 Fast implementation
    let mut rng_128_fast = CtrDrbg::new(
        CtrDrbgVariant::Aes128Fast,
        &entropy_128,
        &nonce[..8], // 8-byte nonce for AES-128
        None,
    )
    .expect("Failed to create AES-128 fast DRBG");

    // AES-128 Small implementation
    let mut rng_128_small =
        CtrDrbg::new(CtrDrbgVariant::Aes128Small, &entropy_128, &nonce[..8], None)
            .expect("Failed to create AES-128 small DRBG");

    // AES-256 Fast implementation
    let mut rng_256_fast = CtrDrbg::new(CtrDrbgVariant::Aes256Fast, &entropy_256, &nonce, None)
        .expect("Failed to create AES-256 fast DRBG");

    // AES-256 Small implementation
    let mut rng_256_small = CtrDrbg::new(CtrDrbgVariant::Aes256Small, &entropy_256, &nonce, None)
        .expect("Failed to create AES-256 small DRBG");

    // Generate random data from each variant
    let mut buffer = [0u8; 16];

    rng_128_fast
        .generate_bytes(&mut buffer)
        .expect("128 fast generation failed");
    rng_128_small
        .generate_bytes(&mut buffer)
        .expect("128 small generation failed");
    rng_256_fast
        .generate_bytes(&mut buffer)
        .expect("256 fast generation failed");
    rng_256_small
        .generate_bytes(&mut buffer)
        .expect("256 small generation failed");
}

/// Example showing reseeding for long-running applications
fn reseeding_example() {
    let initial_entropy = [0x55u8; 32];
    let nonce = [0x66u8; 16];

    let mut rng = CtrDrbg::new_default(&initial_entropy, &nonce).expect("Failed to create DRBG");

    // Generate some initial random data
    let mut buffer = [0u8; 32];
    rng.generate_bytes(&mut buffer)
        .expect("Initial generation failed");

    // After some time or number of generations, reseed for security
    let fresh_entropy = [0x77u8; 32];
    let additional_input = b"periodic_reseed";

    rng.reseed(&fresh_entropy, Some(additional_input))
        .expect("Reseeding failed");

    // Continue generating with improved security properties
    rng.generate_bytes(&mut buffer)
        .expect("Post-reseed generation failed");
}

/// Example showing convenience methods
fn convenience_methods_example() {
    let entropy = [0x88u8; 32];
    let nonce = [0x99u8; 16];

    let mut rng = CtrDrbg::new_default(&entropy, &nonce).expect("Failed to create DRBG");

    // Generate random integers
    let _random_u32 = rng.next_u32().expect("Failed to generate u32");
    let _random_u64 = rng.next_u64().expect("Failed to generate u64");

    // Check algorithm variant and initialization status
    assert_eq!(rng.variant(), CtrDrbgVariant::Aes256Fast);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
