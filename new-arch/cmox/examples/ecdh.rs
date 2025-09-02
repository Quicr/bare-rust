//! ECDH key exchange examples using CMOX library
//!
//! This example demonstrates ECDH key exchange with both NIST curves and Montgomery curves.

#![no_std]
#![no_main]

use cmox::{
    drbg::CtrDrbg,
    ecdh::{Curve, PrivateKey, P256, P384, P521, X25519, X448},
    initialize,
};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main() {
    // Initialize CMOX library
    initialize().unwrap();

    let entropy = [0x42; 32]; // In practice, use real entropy
    let nonce = [0x01; 16];
    let mut rng = CtrDrbg::new_default(&entropy, &nonce).expect("Failed to initialize RNG");

    exchange::<P256>(&mut rng);
    exchange::<P384>(&mut rng);
    exchange::<P521>(&mut rng);
    exchange::<X25519>(&mut rng);
    exchange::<X448>(&mut rng);
}

fn exchange<C: Curve>(rng: &mut CtrDrbg) {
    // Generate key pairs for Alice and Bob
    let (alice_priv, alice_pub) = PrivateKey::<C>::random(rng).unwrap();
    let (bob_priv, bob_pub) = PrivateKey::<C>::random(rng).unwrap();

    // Perform ECDH on both sides
    let alice_shared = alice_priv.exchange(&bob_pub).unwrap();
    let bob_shared = bob_priv.exchange(&alice_pub).unwrap();

    // Shared secrets should be identical
    assert!(alice_shared == bob_shared);
}
