//! Example demonstrating MAC usage with the CMOX crate
//!
//! This example shows how to use HMAC-SHA256 and AES-CMAC for message authentication.

#![no_std]
#![no_main]

use cmox::{
    initialize,
    mac::{AesCmac, HmacSha256},
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().expect("Failed to initialize CMOX");

    // HMAC-SHA256 Example using native API
    {
        let key = b"my_secret_key_for_hmac_sha256_32";
        let data = b"Hello, World!";

        // Create HMAC instance
        let mut hmac = HmacSha256::new_with_key(key).expect("Failed to create HMAC");

        // Update with data to authenticate
        hmac.update_internal(data).expect("Failed to update HMAC");

        // Finalize to get the tag
        let tag = hmac.finalize_internal().expect("Failed to finalize HMAC");

        // tag is now a [u8; 32] array containing the HMAC-SHA256 tag
        // In real applications, you would send both data and tag,
        // then verify on the receiving end

        // Example verification (normally done by receiver)
        let mut hmac_verify = HmacSha256::new_with_key(key).expect("Failed to create HMAC");
        hmac_verify
            .update_internal(data)
            .expect("Failed to update HMAC");
        let verify_tag = hmac_verify
            .finalize_internal()
            .expect("Failed to finalize HMAC");

        // Compare tags (in real code, use constant-time comparison)
        if tag == verify_tag {
            // Authentication successful
        }
    }

    // AES-CMAC Example using native API
    {
        let key = b"1234567890123456"; // 16-byte key for AES-128
        let data = b"Message to authenticate with CMAC";

        // Create CMAC instance
        let mut cmac = AesCmac::new_with_key(key).expect("Failed to create CMAC");

        // Update with data to authenticate
        cmac.update_internal(data).expect("Failed to update CMAC");

        // Finalize to get the tag
        let tag = cmac.finalize_internal().expect("Failed to finalize CMAC");

        // tag is now a [u8; 16] array containing the AES-CMAC tag

        // Example verification (normally done by receiver)
        let mut cmac_verify = AesCmac::new_with_key(key).expect("Failed to create CMAC");
        cmac_verify
            .update_internal(data)
            .expect("Failed to update CMAC");
        let verify_tag = cmac_verify
            .finalize_internal()
            .expect("Failed to finalize CMAC");

        // Compare tags (in real code, use constant-time comparison)
        if tag == verify_tag {
            // Authentication successful
        }
    }

    // Multiple update example
    {
        let key = b"key_for_multiple_updates_32_byte";

        let mut hmac = HmacSha256::new_with_key(key).expect("Failed to create HMAC");

        // You can update multiple times before finalizing
        hmac.update_internal(b"Part 1 of message")
            .expect("Failed to update");
        hmac.update_internal(b" - Part 2 of message")
            .expect("Failed to update");
        hmac.update_internal(b" - Final part")
            .expect("Failed to update");

        let tag = hmac.finalize_internal().expect("Failed to finalize");

        // This produces the same result as computing HMAC on the concatenated message
        let mut hmac_single = HmacSha256::new_with_key(key).expect("Failed to create HMAC");
        hmac_single
            .update_internal(b"Part 1 of message - Part 2 of message - Final part")
            .expect("Failed to update");
        let single_tag = hmac_single.finalize_internal().expect("Failed to finalize");

        if tag == single_tag {
            // Tags match as expected
        }
    }
}
