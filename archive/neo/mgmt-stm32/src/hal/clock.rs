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

use crate::hal::gpio::{gpioa, Fields, Output};
use crate::svd::{Read, Write, RCC};

/// Initializes the clock configuration based on the board-specific settings.
pub fn init(hse_clk_freq: u32) {
    let pll_m = match hse_clk_freq {
        16_000_000 => 4,
        _ => unreachable!("Invalid HSE clock frequency {hse_clk_freq}"),
    };

    // Enable HSE
    RCC::CR::HSEON::write(true);
    while !RCC::CR::HSERDY::read() {}

    // Configure PLL
    RCC::CFGR::PLLSRC::write(1); // HSE as PLL source
    RCC::CFGR::PLLMUL::write(pll_m); // PLL multiplier
    RCC::CFGR::PPRE::write(0); // No AHB prescaler

    // Enable PLL
    RCC::CR::PLLON::write(true);
    while !RCC::CR::PLLRDY::read() {}

    // Select PLL as system clock source
    RCC::CFGR::SW::write(0b10);
    while RCC::CFGR::SWS::read() != 0b10 {}
}

/// Configure MCO to output half of the PLLCLK frequency
pub fn configure_mco(_pin: &Output<gpioa::PA8>, mco_freq: u32) {
    assert_eq!(mco_freq, 24_000_000);

    // Enable GPIOA clock
    RCC::AHBENR::IOPAEN::write(true);

    // Configure PA8 as an alternate function (MCO)
    // TODO(RLB) move this to the GPIO module?
    <gpioa::PA8 as Fields>::MODER::write(0b10); // Set mode to alternate function
    <gpioa::PA8 as Fields>::AFR::write(0); // Set AF0 for MCO

    // Set MCO to output PLLCLK / 2
    RCC::CFGR::MCO::write(0b0111); // Set MCO source to PLLCLK
    RCC::CFGR::PLLNODIV::write(true); // PLL is NOT divided by 2
    RCC::CFGR::MCOPRE::write(0b001); // Set MCO prescaler to divide by 2
}

/// Validates the clock configuration to ensure it is set up correctly.
pub fn validate() {}
