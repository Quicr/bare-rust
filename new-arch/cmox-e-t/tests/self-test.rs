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

    /*
    // Another example for a conditionally enabled test
    #[test]
    fn defmt() {
        defmt::info!("Hello, defmt!");
        assert!(true)
    }

    // Tests can be ignored with the #[ignore] attribute
    #[test]
    #[ignore]
    fn it_works_ignored() {
        assert!(false)
    }

    // A test that fails with a panic
    #[test]
    fn it_fails1() {
        assert!(false)
    }

    // A test that fails with a returned Err(&str)
    #[test]
    fn it_fails2() -> Result<(), &'static str> {
        Err("It failed because ...")
    }

    // Tests can be annotated with #[should_panic] if they are expected to panic
    #[test]
    #[should_panic]
    fn it_passes() {
        assert!(false)
    }

    // This test should panic, but doesn't => it fails
    #[test]
    #[should_panic]
    fn it_fails3() {}

    // Tests can be annotated with #[timeout(<secs>)] to change the default timeout of 60s
    #[test]
    #[timeout(1)]
    fn it_timeouts() {
        loop {} // should run into the 10s timeout
    }
    */
}
