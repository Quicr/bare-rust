//! # Inject Module
//!
//! Allows test program to inject hardware events into the system.
//!

extern crate hal;

#[cfg(feature = "std")]
extern crate std;

pub struct Inject {}

impl crate::inject::Inject {
    #[inline(never)]
    pub fn new() -> Self {
        crate::inject::Inject {}
    }

    #[inline(never)]
    pub fn init(&self) {}

    pub fn ai_button_press(&self) {
        todo!();
    }

    pub fn ai_button_release(&self) {
        todo!();
    }

    pub fn ptt_button_press(&self) {
        todo!();
    }

    pub fn ptt_button_release(&self) {
        todo!();
    }

    pub fn keypress(&self, _key: u8) {
        todo!();
    }

    pub fn data_from_link(&self, _key: &[u8]) {
        todo!();
    }

    pub fn audio_from_mic(&self, _key: &[u16]) {
        todo!();
    }

    pub fn set_battery_percentage(&self, _percentage: u8) {
        todo!();
    }

    // TODO - like to have some sort of trace callback that can be used to
    //        trace data sent to speaker,
    //        sent to link,
    //        set led,
    //        and sent to display.
}
