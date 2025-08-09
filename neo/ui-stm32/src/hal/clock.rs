//! # Clock Module
//!
//! This module provides functions to initialize and validate the clock configuration for the system.
//! It handles setting up the PLL (Phase-Locked Loop), external clock, and various clock dividers.
//!
//! ## Functions
//!
//! - `init`: Initializes the clock configuration based on the board-specific settings.
//! - `validate`: Validates the clock configuration to ensure it is set up correctly.
//!
//! ## Usage
//!
//! The `init` function should be called during system startup to configure the clock. The `validate` function
//! can be used to check if the clock configuration is correct.
//!

use crate::svd::{Read, Write, FLASH, RCC};

fn write_pll_m(pll_m: u8) {
    assert!(pll_m < (1 << 6));
    RCC::PLLCFGR::PLLM5::write((pll_m >> 5) & 0x1 == 1);
    RCC::PLLCFGR::PLLM4::write((pll_m >> 4) & 0x1 == 1);
    RCC::PLLCFGR::PLLM3::write((pll_m >> 3) & 0x1 == 1);
    RCC::PLLCFGR::PLLM2::write((pll_m >> 2) & 0x1 == 1);
    RCC::PLLCFGR::PLLM1::write((pll_m >> 1) & 0x1 == 1);
    RCC::PLLCFGR::PLLM0::write((pll_m >> 0) & 0x1 == 1);
}

fn write_pll_n(pll_n: u8) {
    RCC::PLLCFGR::PLLN7::write((pll_n >> 7) & 0x1 == 1);
    RCC::PLLCFGR::PLLN6::write((pll_n >> 6) & 0x1 == 1);
    RCC::PLLCFGR::PLLN5::write((pll_n >> 5) & 0x1 == 1);
    RCC::PLLCFGR::PLLN4::write((pll_n >> 4) & 0x1 == 1);
    RCC::PLLCFGR::PLLN3::write((pll_n >> 3) & 0x1 == 1);
    RCC::PLLCFGR::PLLN2::write((pll_n >> 2) & 0x1 == 1);
    RCC::PLLCFGR::PLLN1::write((pll_n >> 1) & 0x1 == 1);
    RCC::PLLCFGR::PLLN0::write((pll_n >> 0) & 0x1 == 1);
}

fn write_pll_p(pll_p: u8) {
    assert!(pll_p < (1 << 2));
    RCC::PLLCFGR::PLLP1::write((pll_p >> 1) & 0x1 == 1);
    RCC::PLLCFGR::PLLP0::write((pll_p >> 0) & 0x1 == 1);
}

fn write_pll_q(pll_q: u8) {
    assert!(pll_q < (1 << 4));
    RCC::PLLCFGR::PLLQ3::write((pll_q >> 3) & 0x1 == 1);
    RCC::PLLCFGR::PLLQ2::write((pll_q >> 2) & 0x1 == 1);
    RCC::PLLCFGR::PLLQ1::write((pll_q >> 1) & 0x1 == 1);
    RCC::PLLCFGR::PLLQ0::write((pll_q >> 0) & 0x1 == 1);
}

fn read_pll_n() -> u8 {
    (u8::from(RCC::PLLCFGR::PLLN7::read()) << 7)
        | (u8::from(RCC::PLLCFGR::PLLN6::read()) << 6)
        | (u8::from(RCC::PLLCFGR::PLLN5::read()) << 5)
        | (u8::from(RCC::PLLCFGR::PLLN4::read()) << 4)
        | (u8::from(RCC::PLLCFGR::PLLN3::read()) << 3)
        | (u8::from(RCC::PLLCFGR::PLLN2::read()) << 2)
        | (u8::from(RCC::PLLCFGR::PLLN1::read()) << 1)
        | (u8::from(RCC::PLLCFGR::PLLN0::read()) << 0)
}

fn read_pll_p() -> u8 {
    (u8::from(RCC::PLLCFGR::PLLP1::read()) << 1) | (u8::from(RCC::PLLCFGR::PLLP0::read()) << 0)
}

fn read_pll_q() -> u8 {
    (u8::from(RCC::PLLCFGR::PLLQ3::read()) << 3)
        | (u8::from(RCC::PLLCFGR::PLLQ2::read()) << 2)
        | (u8::from(RCC::PLLCFGR::PLLQ1::read()) << 1)
        | (u8::from(RCC::PLLCFGR::PLLQ0::read()) << 0)
}

/// Initializes the clock configuration based on the board-specific settings.
pub fn init(hse_clk_freq: u32) {
    let pll_m = match hse_clk_freq {
        16_000_000 => 8,
        24_000_000 => 12,
        _ => unreachable!("Invalid HSE clock frequency {hse_clk_freq}"),
    };

    // Setup flash wait states and cache
    // XXX(fluffy): If voltage is changed, need to change this
    FLASH::ACR::LATENCY::write(5);
    FLASH::ACR::PRFTEN::write(true);
    FLASH::ACR::ICEN::write(true);
    FLASH::ACR::DCEN::write(true);

    // Enable HSE
    RCC::CR::HSEON::write(true);
    while !RCC::CR::HSERDY::read() {}

    // Set up main PLL timing for external HSE
    const PLL_N: u8 = 168;
    const PLL_P: u8 = 2;
    const PLL_Q: u8 = 4;
    write_pll_m(pll_m);
    write_pll_n(PLL_N);
    write_pll_p(PLL_P);
    write_pll_q(PLL_Q);

    // Select HSE
    RCC::PLLCFGR::PLLSRC::write(true);

    // Enable PLL
    RCC::CR::PLLON::write(true);
    while !RCC::CR::PLLRDY::read() {}

    // Set up clock usage and dividers
    RCC::CFGR::HPRE::write(0b0000); // sys clock div = 1
    RCC::CFGR::PPRE1::write(0b101); // APB1 clock div = 4
    RCC::CFGR::PPRE2::write(0b100); // APB2 clock div = 2

    // Switch clock to PLL
    RCC::CFGR::SW1::write(true);
    RCC::CFGR::SW0::write(true);
    while !RCC::CFGR::SWS1::read() || RCC::CFGR::SWS0::read() {}
}

/// Validates the clock configuration to ensure it is set up correctly.
pub fn validate() {
    // Check if HSE is ready
    assert!(RCC::CR::HSERDY::read(), "HSE not ready");

    // Check if PLL is ready
    assert!(RCC::CR::PLLRDY::read(), "PLL not ready");

    // Check if PLL source is HSE
    assert!(RCC::PLLCFGR::PLLSRC::read(), "PLL source is not HSE");

    // Check if the PLL parameters are set correctly
    assert_eq!(read_pll_n(), 168, "PLL N not set to 168");
    assert_eq!(read_pll_p(), 2, "PLL P not set to 2");
    assert_eq!(read_pll_q(), 4, "PLL Q not set to 4");

    // Check if system clock mux is set to PLL
    assert!(
        RCC::CFGR::SWS1::read() && !RCC::CFGR::SWS0::read(),
        "System clock not set to PLL"
    );

    // Check AHB prescaler
    assert_eq!(
        RCC::CFGR::HPRE::read(),
        0b0000,
        "AHB prescaler not set to 1"
    );
    assert_eq!(
        RCC::CFGR::PPRE1::read(),
        0b101,
        "APB1 prescaler not set to 4"
    );
    assert_eq!(
        RCC::CFGR::PPRE2::read(),
        0b100,
        "APB2 prescaler not set to 2"
    );
}
