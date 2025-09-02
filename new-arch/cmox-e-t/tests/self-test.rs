// To do performance profiling:
// probe-rs profile --chip STM32F072CBTx --duration 10 target/thumbv6m-none-eabi/debug/deps/example_test-16bf42a63a1cce53 naive
// probe-rs profile --chip STM32F072CBTx --duration 10 target/thumbv6m-none-eabi/debug/deps/example_test-16bf42a63a1cce53 pcsr

#![no_std]
#![no_main]

// This links the HAL so that reset vectors, etc. are populated
use embassy_stm32 as _;

// This ensures that the defmt library is linked, so that the test framework can use it
use defmt as _;

#[embedded_test::tests(setup=rtt_target::rtt_init_defmt!())]
mod unit_tests {
    use defmt::unwrap;
    use embassy_stm32::crc::{self, Crc};

    // The init function enables the CRC peripheral.  This seems to be needed for some
    // cryptographic functions, not sure why.
    #[init]
    fn init() {
        let p = embassy_stm32::init(Default::default());
        let _ = Crc::new(
            p.CRC,
            unwrap!(crc::Config::new(
                crc::InputReverseConfig::Byte,
                true,
                crc::PolySize::Width32,
                0xFFFFFFFF,
                0x04C11DB7
            )),
        );
    }

    #[test]
    fn version() {
        let version = cmox::version();
        assert_eq!(version, 0x040000B1);
    }

    #[test]
    fn initialize() {
        assert!(cmox::initialize().is_ok());
        assert!(cmox::is_initialized());
        assert!(cmox::finalize().is_ok());
        assert!(!cmox::is_initialized());
    }

    #[test]
    fn constant_time_eq() {
        use cmox::utils::constant_time_eq;

        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(!constant_time_eq(b"hell", b"hello"));
    }

    #[test]
    #[ignore] // XXX(RLB) Complains about reseed-needed
    fn drbg() {
        use cmox::drbg::*;

        let mut drbg = unwrap!(CtrDrbg::new_default(&[0x42; 32], &[0x43; 5]));

        let mut output = [0_u8; 1024];
        unwrap!(drbg.generate(&mut output, None));

        // Test that the number of zero bytes in the output is roughly what you would expect from
        // andom output (within a factor of two)
        let zeros = output.iter().filter(|&x| *x == 0).count();
        assert!(zeros < output.len() / 256 * 2);
    }

    #[test]
    #[ignore] // XXX(RLB) Unexplained halt
    fn hash() {
        use cmox::hash::*;
        use digest::Digest;

        let h = Sha256::digest(b"hello, world");

        // Test that there are no more than 2 zero bytes in the hash output
        let zeros = h.iter().filter(|&x| *x == 0).count();
        assert!(zeros <= 2);
    }

    #[test]
    #[ignore] // XXX(RLB) Unexplained halt
    fn mac() {
        use cmox::mac::*;
        use digest::{Key, Mac};

        let key = Key::<HmacSha256>::from_slice(&[0xA0; 32]);
        let h = HmacSha256::new(key)
            .chain_update(b"hello, world")
            .finalize();

        // Test that there are no more than 2 zero bytes in the hash output
        let zeros = h.into_bytes().iter().filter(|&x| *x == 0).count();
        assert!(zeros <= 2);
    }

    #[test]
    #[ignore] // XXX(RLB) Unexplained halt
    fn aead() {
        use aead::{AeadInPlace, Key, KeyInit, Nonce};
        use cmox::aead::*;

        let key = Key::<Aes128FastGcmFast>::from_slice(&[0x42; 16]);
        let nonce = Nonce::<Aes128FastGcmFast>::from_slice(&[0x01; 12]);
        let aad = b"associated data";

        let cipher = Aes128FastGcmFast::new(&key);

        let original = [0xA0; 256];

        let mut encrypted = original.clone();
        let tag = unwrap!(cipher
            .encrypt_in_place_detached(nonce, aad, &mut encrypted)
            .map_err(|_| ()));

        let mut decrypted = encrypted.clone();
        unwrap!(cipher
            .decrypt_in_place_detached(nonce, aad, &mut decrypted, &tag)
            .map_err(|_| ()));

        assert_ne!(original, encrypted);
        assert_eq!(original, decrypted);
    }

    #[test]
    #[ignore] // XXX(RLB) Unexplained halt
    fn ecdh() {
        use cmox::drbg::CtrDrbg;
        use cmox::ecdh::*;

        let entropy = [0x42; 32];
        let nonce = [0x01; 128];
        let mut rng = unwrap!(CtrDrbg::new_default(&entropy, &nonce));

        let (alice_priv, alice_pub) = PrivateKey::<P256>::random(&mut rng).unwrap();
        let (bob_priv, bob_pub) = PrivateKey::<P256>::random(&mut rng).unwrap();

        let alice_shared = alice_priv.exchange(&bob_pub).unwrap();
        let bob_shared = bob_priv.exchange(&alice_pub).unwrap();

        assert!(alice_shared == bob_shared);
    }

    #[test]
    #[ignore] // XXX(RLB) Unexplained halt
    fn signature() {
        // ECDSA
        {
            use cmox::drbg::CtrDrbg;
            use cmox::signature::ecdsa::*;
            use signature::{RandomizedSigner, Verifier};

            let entropy = [0x42; 32];
            let nonce = [0x01; 128];
            let mut rng = unwrap!(CtrDrbg::new_default(&entropy, &nonce));

            let message = b"Hello, world!";
            let (private_key, public_key) = unwrap!(PrivateKey::<P256>::random(&mut rng));
            let signature = private_key.sign_with_rng(&mut rng, message);
            unwrap!(public_key.verify(message, &signature).map_err(|_| ()));
        }

        // EdDSA
        {
            use cmox::drbg::CtrDrbg;
            use cmox::signature::eddsa::*;
            use signature::{Signer, Verifier};

            let entropy = [0x42; 32];
            let nonce = [0x01; 128];
            let mut rng = unwrap!(CtrDrbg::new_default(&entropy, &nonce));

            let message = b"Hello, world!";
            let (private_key, public_key) = unwrap!(PrivateKey::<Ed25519>::random(&mut rng));
            let signature = private_key.sign(message);
            unwrap!(public_key.verify(message, &signature).map_err(|_| ()));
        }
    }
}
