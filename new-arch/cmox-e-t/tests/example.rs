// To do performance profiling:
// probe-rs profile --chip STM32F072CBTx --duration 10 target/thumbv6m-none-eabi/debug/deps/example_test-16bf42a63a1cce53 naive
// probe-rs profile --chip STM32F072CBTx --duration 10 target/thumbv6m-none-eabi/debug/deps/example_test-16bf42a63a1cce53 pcsr

#![no_std]
#![no_main]

// This links the HAL so that reset vectors, etc. are populated
use stm32f0xx_hal as _;

// This ensures that the defmt library is linked, so that the test framework can use it
use defmt as _;

#[embedded_test::tests(setup=rtt_target::rtt_init_defmt!())]
mod unit_tests {
    use cmox_sys::cmox_getInfos;
    use core::mem::MaybeUninit;
    use defmt::info;

    #[test]
    fn version() {
        let mut info = unsafe { MaybeUninit::zeroed().assume_init() };
        unsafe { cmox_getInfos(&mut info) };

        info!("version = {}", info.version);
        info!("build = {:?}", info.build);

        assert_eq!(info.version, 1);
        assert_eq!(info.build, [0, 0, 0, 0, 0, 0, 0]);
    }

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
}
