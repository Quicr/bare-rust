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

use core::ptr;

use super::cpu;
use super::cpu::*;

#[allow(unused_imports)]
use super::gpio;

#[cfg(all(feature = "stm32f072", feature = "stm32f405"))]
compile_error!("Must specify only a single CPU type featre");

#[cfg(feature = "stm32f072")]
#[inline(never)]
/// Initializes the clock configuration based on the board-specific settings.
pub fn init(hse_clk_freq: u32) {
    let pll_m: u32;
    match hse_clk_freq {
        16_000_000 => {
            // pll_m = 0b0001; // this is a x3 pll mult for 48MHz
            // pll_m = 0b1010; // hack x12 TODO - fails to boot
            pll_m = 0b0100; // hack x6 TODO
        }

        _ => {
            panic!("HSE Clk Freq not supported");
        }
    }

    // Enable HSE
    cpu::write!(RCC.cr[HSEON;1], 1);

    // Wait for HSE to be ready
    while cpu::read!(RCC.cr[HSERDY;1]) == 0 {}

    // Configure PLL
    cpu::write!(RCC.cfgr[PLLSRC;2], 1); // HSE as PLL source
    cpu::write!(RCC.cfgr[PLLMUL;4], pll_m); // PLL multiplier
    cpu::write!(RCC.cfgr[PPRE;3], 0b000); // No AHB prescaler

    // Enable PLL
    cpu::write!(RCC.cr[PLLON;1], 1);

    // Wait for PLL to be ready
    while cpu::read!(RCC.cr[PLLRDY;1]) == 0 {}

    // Select PLL as system clock source
    cpu::write!(RCC.cfgr[SW;2], 0b10); // PLL as system clock source

    // Wait for PLL to be used as system clock source
    while cpu::read!(RCC.cfgr[SWS;2]) != 0b10 {}
}

// Configures MCO to output half the PLLCLK frequency.
#[cfg(feature = "stm32f072")]
#[inline(never)]
pub fn configure_mco(pin: super::gpio::Pin, mco_freq: u32) {
    // TODO
    assert!(pin.0 == GPIOA as *mut cpu::GpioReg);
    assert!(pin.1 >= 8);

    let pn = pin.1;

    assert!(mco_freq == 24_000_000);

    // Enable GPIOA clock
    cpu::write!(RCC.ahbenr[IOPAEN;1], 1);

    // Configure PA8 as alternate function (MCO)
    cpu::write!(GPIOA.moder[pn*2;2], 0b10); // Set mode to alternate function
    cpu::write!(GPIOA.afrh[(pn-8)*4;4], 0); // Set AF0 for MCO

    // Configure MCO to output PLLCLK/2
    cpu::write!(RCC.cfgr[MCO;4], 0b0111); // Set MCO source to PLLCLK
                                          //cpu::write!(RCC.cfgr[MCO;4], 0b0100); // Set MCO source to SYSCLK

    cpu::write!(RCC.cfgr[PLLNODIV;1], 1); // PLL is NOT devided by 2

    //cpu::write!(RCC.cfgr[MCOPRE;3], 0b000); // MCO No prescaler
    cpu::write!(RCC.cfgr[MCOPRE;3], 0b001); // Set MCO prescaler to divide by 2
                                            //cpu::write!(RCC.cfgr[MCOPRE;3], 0b011); // MCO prescaler set to divide by 8
}

#[cfg(feature = "stm32f072")]
#[inline(never)]
pub fn validate() {}

#[cfg(feature = "stm32f405")]
#[inline(never)]
/// Initializes the clock configuration based on the board-specific settings.
pub fn init(_hse_clk_freq: u32) {
    #[cfg(feature = "std")]
    let clk_freq: u32 = 0;
    #[cfg(not(feature = "std"))]
    let clk_freq: u32 = _hse_clk_freq;

    let pll_m: u32;
    match clk_freq {
        0 => {
            pll_m = 8;
        }

        16_000_000 => {
            pll_m = 8;
        }

        24_000_000 => {
            pll_m = 12;
        }

        _ => {
            panic!("HSE Clk Freq not supported");
        }
    }

    if clk_freq != 0 {
        cpu::write!(RCC.cfgr[SWS0;2], 0b10);
        cpu::write!(RCC.cr[PLLRDY;1], 1);
        cpu::write!(RCC.cr[HSERDY;1], 1);
        cpu::write!(RCC.pllcfgr[PLLSRC;1], 1);
        cpu::write!(RCC.pllcfgr[PLLM0;6], pll_m);
    }

    // setup flash wait states and cache
    {
        // set latency to 5 wait states - NOTE, if voltage is changed, need to change this
        cpu::write!(FLASH.acr[LATENCY;3], 5);

        // enable data, instruction, prefetch cache
        cpu::write!(FLASH.acr[PRFTEN;1], 1);
        cpu::write!(FLASH.acr[ICEN;1], 1);
        cpu::write!(FLASH.acr[DCEN;1], 1);
    }

    // set up external clock and PLL
    {
        // enable HSE
        cpu::write!(RCC.cr[HSEON;1], 1);

        // wait for HSE to be ready
        if clk_freq != 0 {
            while (cpu::read!(RCC.cr[HSERDY;1]) != 1) {}
        }

        // setup main PLL timing for external HSE
        let pll_m: u32 = pll_m;

        let pll_n: u32 = 168;
        let pll_q: u32 = 4;

        assert!(pll_q >= 2);
        assert!(pll_q <= 0xF);
        assert!(pll_n >= 50);
        assert!(pll_n <= 432);
        assert!(pll_m >= 2);
        assert!(pll_m <= 63);

        cpu::write!(RCC.pllcfgr[PLLQ0;4], pll_q );
        cpu::write!(RCC.pllcfgr[PLLM0;5], pll_m );
        cpu::write!(RCC.pllcfgr[PLLN0;9], pll_n );

        // set main division factor to 2
        cpu::write!(RCC.pllcfgr[PLLP0;2], 0b00);

        // select HSE
        cpu::write!(RCC.pllcfgr[PLLSRC;1], 0b1);

        // enable PLL
        cpu::write!(RCC.cr[PLLON;1], 0b1);
        // wait for PLL to be ready
        if clk_freq != 0 {
            while (cpu::read!(RCC.cr[PLLRDY;1]) != 1) {}
        }

        // setup clock usage and dividers
        // sys clock div 1
        cpu::write!(RCC.cfgr[HPRE;4], 0b0000);
        // APB1 Clk Div = 4
        cpu::write!(RCC.cfgr[PPRE1;3], 0b101);
        // APB2 Clk Div = 2
        cpu::write!(RCC.cfgr[PPRE2;3], 0b100);

        // switch clock to PLL
        cpu::write!(RCC.cfgr[SW0;2], 0b10 );

        // wait for clock to switch to PLL
        if clk_freq != 0 {
            while (cpu::read!(RCC.cfgr[SWS0;2]) != 0b10) {}
        }
    }
}

#[cfg(feature = "stm32f405")]
#[inline(never)]
/// Validates the clock configuration to ensure it is set up correctly.
pub fn validate() {
    // Check if HSE is ready
    //if board::info::HAS_RCC {
    //    if cpu::read!(RCC.cr[HSERDY;1]) != 1 {
    //        panic!("HSE not ready");
    //    }
    //}

    // Check if PLL is ready
    //if board::info::HAS_RCC {
    //    if cpu::read!(RCC.cr[PLLRDY;1]) != 1 {
    //        panic!("PLL not ready");
    //    }
    //}

    // Check if PLL source is HSE
    // seems like this can not be read after it is enabled
    //if cpu::read!(RCC.pllcfgr[PLLSRC;1]) != 1 {
    //    panic!("PLL source not set to HSE");
    //}

    // Check if PLL M is set correctly
    // seems like this can not be read after it is enabled
    //if read!(RCC.pllcfgr[PLLM0;6]) != board::info::CLOCK_PLL_M {
    //    panic!("PLL M not set correctly");
    //}

    // Check if PLL N is set to 168
    if read!(RCC.pllcfgr[PLLN0;9]) != 168 {
        panic!("PLL N not set to 168");
    }

    // Check if PLL P is set to 2
    if read!(RCC.pllcfgr[PLLP0;2]) != 0 {
        panic!("PLL P not set to 2");
    }

    // Check if PLL Q is set to 4
    if read!(RCC.pllcfgr[PLLQ0;4]) != 4 {
        panic!("PLL Q not set to 4");
    }

    // Check if system clock mux is set to PLL
    if cpu::read!(RCC.cfgr[SWS0;2]) != 0b10 {
        panic!("System clock not set to PLL");
    }

    // Check AHB prescaler
    if cpu::read!(RCC.cfgr[HPRE;4]) != 0b0000 {
        panic!("AHB prescaler not set to 1");
    }

    // Check APB1 prescaler
    if cpu::read!(RCC.cfgr[PPRE1;3]) != 0b101 {
        panic!("APB1 prescaler not set to 4");
    }

    // Check APB2 prescaler
    if cpu::read!(RCC.cfgr[PPRE2;3]) != 0b100 {
        panic!("APB2 prescaler not set to 2");
    }
}
