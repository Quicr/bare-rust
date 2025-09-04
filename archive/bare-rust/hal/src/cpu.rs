//! # CPU Module
//!
//! This module provides low-level access to various CPU registers and peripherals.
//! It includes definitions and functions for interacting
//! with the Flash, RCC, NVIC, GPIO, USART, and TIM registers.
//! Additionally, it provides macros for reading and writing to these registers.
//!
//! ## Structures
//!
//! - `FlashReg`: Represents the Flash memory interface registers.
//! - `RccReg`: Represents the Reset and Clock Control registers.
//! - `NVICReg`: Represents the Nested Vectored Interrupt Controller registers.
//! - `GpioReg`: Represents the General-Purpose Input/Output registers.
//! - `UsartReg`: Represents the Universal Synchronous/Asynchronous Receiver/Transmitter registers.
//! - `TimAdvReg`: Represents the Advanced Control Timer registers.
//! - `TimGenReg`: Represents the General Purpose Timer registers.
//!
//! ## Constants
//!
//! - `FLASH`: Base address for the Flash registers.
//! - `RCC`: Base address for the RCC registers.
//! - `NVIC`: Base address for the NVIC registers.
//! - `GPIOA`, `GPIOB`, `GPIOC`: Base addresses for the GPIO registers.
//! - `USART1`: Base address for the USART1 registers.
//! - `TIM1`, `TIM2`: Base addresses for the TIM1 and TIM2 registers.
//!
//! ## Macros
//!
//! - `write!`: Macro for writing to a register.
//! - `read!`: Macro for reading from a register.
//!
//! ## Functions
//!
//! - `init`: Initializes the simulator memory (for `board-sim` feature).
//!
//! The following functions should not be use and instead use the macros.
//!
//! - `update_reg`: Updates a register with a masked value.
//! - `write_reg`: Writes a value to a register.
//! - `read_reg`: Reads a value from a register.
//!
//! ## Usage
//!
//! This module is intended for low-level hardware interaction and should be used with caution.
//! It provides direct access to hardware registers, which can lead to undefined
//! behavior if used incorrectly.
//!

#[cfg(feature = "stm32f405")]
use super::svd_stm32f405 as svd;

#[cfg(feature = "stm32f072")]
use super::svd_stm32f0x2 as svd;

#[cfg(any(feature = "stm32f072", feature = "stm32f405"))]
pub use svd::*;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::Mutex;

#[cfg(feature = "stm32f405")]
#[repr(C)]
pub struct NVICReg {
    pub iser: [u32; 8], // Interrupt Set-Enable Registers
    pub icer: [u32; 8], // Interrupt Clear-Enable Registers
    pub ispr: [u32; 8], // Interrupt Set-Pending Registers
    pub icpr: [u32; 8], // Interrupt Clear-Pending Registers
    pub iabr: [u32; 8], // Interrupt Active Bit Registers
    pub ipr: [u32; 60], // Interrupt Priority Registers
    pub stir: u32,      // Software Trigger Interrupt Register
}

#[cfg(feature = "stm32f405")]
pub const NVIC: *mut NVICReg = 0xE000_E100 as *mut NVICReg;

#[inline(always)]
//#[inline(never)]
pub fn update_reg(addr: *mut u32, mask: u32, val: u32) {
    if cfg!(feature = "std") {
        let mut v: u32 = read_reg(addr);
        v &= !mask;
        v |= val;
        write_reg(addr, v);
    } else {
        unsafe {
            let mut v: u32 = core::ptr::read_volatile(addr);
            v &= !mask;
            v |= val;
            core::ptr::write_volatile(addr, v);
        }
    }
}

#[cfg(not(feature = "std"))]
#[inline(always)]
pub fn write_reg(addr: *mut u32, val: u32) {
    unsafe {
        core::ptr::write_volatile(addr, val);
    }
}

#[cfg(feature = "std")]
#[inline(always)]
//#[inline(never)]
pub fn write_reg(addr: *mut u32, val: u32) {
    unsafe {
        if let Some(ref map_mutex) = SIM {
            let mut map = map_mutex.lock().unwrap();
            map.insert(addr, val);
        }
    }
}

#[cfg(not(feature = "std"))]
#[inline(always)]
pub fn read_reg(addr: *mut u32) -> u32 {
    unsafe { core::ptr::read_volatile(addr) }
}

#[cfg(feature = "std")]
#[inline(always)]
//#[inline(never)]
pub fn read_reg(addr: *mut u32) -> u32 {
    unsafe {
        if let Some(ref map_mutex) = SIM {
            let map = map_mutex.lock().unwrap();
            if let Some(value) = map.get(&addr) {
                let my_value: u32 = *value;
                my_value
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[macro_export]
macro_rules! write {
    ( $x:ident.$y:ident[$z:ident;$w:expr],  $data:expr  ) => {
        let offset: u8 = $x::$y::$z;
        //let offset = concat_idents!($x, _, $y, _, $z);
        let mut mask: u32 = (1u32 << $w) - 1;
        let mut val: u32 = $data & mask;
        mask = mask << offset;
        val = val << offset;
        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y);
            cpu::update_reg(addr, mask, val);
        }
    };

    ( $x:ident.$y:ident[$z:expr;$w:expr],  $data:expr  ) => {{
        let offset: u8 = $z;
        let mut mask: u32 = (1u32 << $w) - 1;
        let mut val: u32 = $data & mask;
        mask = mask << offset;
        val = val << offset;
        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y);
            cpu::update_reg(addr, mask, val);
        }
    }};

    ( $x:ident.$y:ident[$z:expr],  $data:expr  ) => {{
        let val: u32 = $data;
        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y[$z]);
            cpu::write_reg(addr, val);
        }
    }};

    ( $x:ident.$y:ident ,  $data:expr  ) => {
        let val: u32 = $data;
        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y);
            cpu::write_reg(addr, val);
        }
    };
}

pub(crate) use write;

#[macro_export]
macro_rules! read {
    ( $x:ident.$y:ident[$z:ident;$w:expr] ) => {{
        let offset: u8 = $x::$y::$z;
        let mask: u32 = (1u32 << $w) - 1;
        let mut val: u32;

        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y);
            val = cpu::read_reg(addr);
        }
        val = val >> offset;
        val = val & mask;
        val
    }};
    ( $x:ident.$y:ident[$z:expr;$w:expr] ) => {{
        let offset: u8 = $z;
        let mask: u32 = (1u32 << $w) - 1;
        let mut val: u32;

        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y);
            val = cpu::read_reg(addr);
        }
        val = val >> offset;
        val = val & mask;
        val
    }};
    ( $x:ident.$y:ident ) => {{
        let val: u32;
        unsafe {
            let addr = ptr::addr_of_mut!((*$x).$y);
            val = cpu::read_reg(addr);
        }
        val
    }};
}

pub(crate) use read;

#[cfg(feature = "std")]
static mut SIM: Option<Mutex<&'static mut HashMap<*mut u32, u32>>> = None;

#[cfg(not(feature = "std"))]
fn init_sim() {}

#[cfg(feature = "std")]
fn init_sim() {
    // initialize the simulator memory
    static mut MEM: Option<HashMap<*mut u32, u32>> = None;
    #[allow(static_mut_refs)]
    unsafe {
        MEM = Some(HashMap::new());
        SIM = Some(Mutex::new(MEM.as_mut().unwrap()));
    }
}

pub fn init() {
    init_sim();
}

#[cfg(test)]
mod tests {
    //use core::ptr;

    //use crate::board;
    //use crate::cpu;
    // crate::cpu::*;
}
