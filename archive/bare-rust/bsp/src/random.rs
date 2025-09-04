//! # Random Module

extern crate hal;

#[cfg(feature = "std")]
extern crate std;

pub struct Random {}

impl crate::random::Random {
    #[inline(never)]
    pub fn new() -> Self {
        crate::random::Random {}
    }

    #[inline(never)]
    pub fn init(&self) {}

    pub fn u128(&self) -> u128 {
        todo!();
    }
}
