#![no_std]
#![no_main]

// This links the HAL so that reset vectors, etc. are populated
use embassy_stm32 as _;
use embassy_stm32::{bind_interrupts, peripherals, rng};

// This ensures that the defmt library is linked, so that the test framework can use it
use defmt as _;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[cfg(test)]
#[embedded_test::tests(setup=rtt_target::rtt_init_defmt!())]
mod unit_tests {
    use super::Irqs;
    use defmt::unwrap;
    use embassy_stm32::{crc::Crc, peripherals::RNG, rng::Rng};

    // The init function enables the CRC peripheral.  This seems to be needed for some
    // cryptographic functions, not sure why.
    #[init]
    fn init() -> Rng<'static, RNG> {
        let config = {
            use embassy_stm32::rcc::*;

            let mut config = embassy_stm32::Config::default();
            config.rcc.hsi = true;
            config.rcc.sys = Sysclk::PLL1_P;
            config.rcc.pll_src = PllSource::HSI;
            config.rcc.pll = Some(Pll {
                prediv: PllPreDiv::DIV8,
                mul: PllMul::MUL168,
                divp: Some(PllPDiv::DIV2),
                divq: Some(PllQDiv::DIV7),
                divr: None,
            });
            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV4;
            config.rcc.apb2_pre = APBPrescaler::DIV2;
            config.rcc.ls = LsConfig {
                rtc: RtcClockSource::LSI,
                lsi: true,
                lse: None,
            };

            // XXX(RLB): The clocks should be driven off of HSE, but the Embassy clock init code
            // hangs if I configure HSE, presumably waiting for `hserdy`.  The above configuration
            // is a hack that produces the same output frequencies using the 16Mhz HSI clock as a
            // base and dividing it down further (2Mhz = 16Mhz / M=8 vs. 6Mhz / M=3).  That seems
            // to be good enough to get the tests going.
            /*
            config.rcc.hse = Some(Hse {
                freq: Hertz(6_000_000),
                mode: HseMode::Bypass,
            });
            config.rcc.sys = Sysclk::PLL1_P;
            config.rcc.pll_src = PllSource::HSE;
            config.rcc.pll = Some(Pll {
                prediv: PllPreDiv::DIV3,
                mul: PllMul::MUL168,
                divp: Some(PllPDiv::DIV2),
                divq: Some(PllQDiv::DIV7),
                divr: None,
            });
            */

            config
        };

        let p = embassy_stm32::init(config);
        let _ = Crc::new(p.CRC);
        Rng::new(p.RNG, Irqs)
    }

    #[test]
    fn drbg(mut rng: Rng<'static, RNG>) {
        let mut output = [0_u8; 1024];
        rng.fill_bytes(&mut output);

        // Test that the number of zero bytes in the output is roughly what you would expect from
        // random output (within a factor of two)
        let zeros = output.iter().filter(|&x| *x == 0).count();
        assert!(zeros < output.len() / 256 * 2);
    }

    #[test]
    fn hash() {
        use digest::Digest;
        use sha2::Sha256;

        let h = Sha256::digest(b"hello, world");

        // Test that there are no more than 2 zero bytes in the hash output
        let zeros = h.iter().filter(|&x| *x == 0).count();
        assert!(zeros <= 2);
    }

    #[test]
    fn mac() {
        use digest::{Key, Mac};
        use hmac::Hmac;
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let key = Key::<HmacSha256>::from_slice(&[0xA0; 64]);
        let h = HmacSha256::new(key)
            .chain_update(b"hello, world")
            .finalize();

        // Test that there are no more than 2 zero bytes in the hash output
        let zeros = h.into_bytes().iter().filter(|&x| *x == 0).count();
        assert!(zeros <= 2);
    }

    #[test]
    fn aead() {
        use aead::{AeadInPlace, Key, KeyInit, Nonce};
        use aes_gcm::Aes128Gcm;

        let key = Key::<Aes128Gcm>::from_slice(&[0x42; 16]);
        let nonce = Nonce::<Aes128Gcm>::from_slice(&[0x01; 12]);
        let aad = b"associated data";

        let cipher = Aes128Gcm::new(&key);

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
    fn ecdh(mut rng: Rng<'static, RNG>) {
        // P-256
        {
            use p256::{ecdh::diffie_hellman, SecretKey};

            let alice_priv = SecretKey::random(&mut rng);
            let alice_pub = alice_priv.public_key();

            let bob_priv = SecretKey::random(&mut rng);
            let bob_pub = bob_priv.public_key();

            let alice_shared = diffie_hellman(alice_priv.to_nonzero_scalar(), bob_pub.as_affine());
            let bob_shared = diffie_hellman(bob_priv.to_nonzero_scalar(), alice_pub.as_affine());

            assert!(alice_shared.raw_secret_bytes() == bob_shared.raw_secret_bytes());
        }

        // X25519
        {
            use x25519_dalek::{EphemeralSecret, PublicKey};

            let alice_secret = EphemeralSecret::random_from_rng(&mut rng);
            let alice_public = PublicKey::from(&alice_secret);

            let bob_secret = EphemeralSecret::random_from_rng(&mut rng);
            let bob_public = PublicKey::from(&bob_secret);

            let alice_shared = alice_secret.diffie_hellman(&bob_public);
            let bob_shared = bob_secret.diffie_hellman(&alice_public);

            assert!(alice_shared.as_bytes() == bob_shared.as_bytes());
        }
    }

    #[test]
    fn signature(mut rng: Rng<'static, RNG>) {
        // ECDSA
        {
            use p256::ecdsa::{Signature, SigningKey};
            use signature::{RandomizedSigner, Verifier};

            let private_key = SigningKey::random(&mut rng);
            let public_key = private_key.verifying_key();

            let message = b"Hello, world!";
            let signature: Signature = private_key.sign_with_rng(&mut rng, message);
            assert!(public_key.verify(message, &signature).is_ok());
        }

        // EdDSA
        {
            use ed25519_dalek::SigningKey;
            use signature::{Signer, Verifier};

            let private_key = SigningKey::generate(&mut rng);
            let public_key = private_key.verifying_key();

            let message = b"Hello, world!";
            let signature = private_key.sign(message);
            assert!(public_key.verify(message, &signature).is_ok());
        }
    }
}
