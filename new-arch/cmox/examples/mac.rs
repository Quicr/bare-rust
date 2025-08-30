//! Example demonstrating MAC usage with the CMOX crate
//!
//! This example shows how to use HMAC-SHA256 and AES-CMAC for message authentication.

#![no_std]
#![no_main]

use cmox::{initialize, mac::HmacSha256};
use digest::Mac;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // HMAC-SHA256 Example
    {
        let key = b"my_secret_key_for_hmac_sha256_32";
        let data = b"Hello, World!";

        // Create HMAC instance
        let mut hmac = HmacSha256::new_from_slice(key).expect("Failed to create HMAC");

        // Update with data to authenticate
        hmac.update(data);

        // Finalize to get the tag
        let tag = hmac.finalize();

        // tag is now a [u8; 32] array containing the HMAC-SHA256 tag
        // In real applications, you would send both data and tag,
        // then verify on the receiving end

        // Example verification (normally done by receiver)
        let mut hmac_verify = HmacSha256::new_from_slice(key).expect("Failed to create HMAC");
        hmac_verify.update(data);
        let verify_tag = hmac_verify.finalize();

        // Compare tags (in real code, use constant-time comparison)
        if tag == verify_tag {
            // Authentication successful
        }
    }

    // Multiple update example
    {
        let key = b"key_for_multiple_updates_32_byte";

        let mut hmac = HmacSha256::new_from_slice(key).expect("Failed to create HMAC");

        // You can update multiple times before finalizing
        hmac.update(b"Part 1 of message");
        hmac.update(b" - Part 2 of message");
        hmac.update(b" - Final part");

        let tag = hmac.finalize();

        // This produces the same result as computing HMAC on the concatenated message
        let mut hmac_single = HmacSha256::new_from_slice(key).expect("Failed to create HMAC");
        hmac_single.update(b"Part 1 of message - Part 2 of message - Final part");
        let single_tag = hmac_single.finalize();

        if tag == single_tag {
            // Tags match as expected
        }
    }
}
