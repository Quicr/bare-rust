use embassy_stm32::gpio::{Output, Pull, Speed};
use embassy_time::{Duration, Timer};

use crate::gpio::{NetControl, UiControl};

/// Power cycle a GPIO pin (set low, delay, set high)
async fn power_cycle(pin: &mut Output<'static>, delay_ms: u64) {
    pin.set_low();
    Timer::after(Duration::from_millis(delay_ms)).await;
    pin.set_high();
}

/// Control functions for NET chip
impl NetControl {
    /// Put NET chip into bootloader mode
    pub async fn bootloader_mode(&mut self) {
        power_cycle(&mut self.nrst, 10).await;

        // Bring boot low for ESP bootloader mode
        self.boot.set_low();

        // Power cycle
        power_cycle(&mut self.nrst, 10).await;
    }

    /// Put NET chip into normal mode
    pub async fn normal_mode(&mut self) {
        self.boot.set_high();

        // Power cycle
        power_cycle(&mut self.nrst, 10).await;
    }

    /// Hold NET chip in reset
    pub async fn hold_in_reset(&mut self) {
        self.boot.set_high();

        // Reset and hold
        self.nrst.set_low();
        Timer::after(Duration::from_millis(100)).await;
    }
}

/// Control functions for UI chip
impl UiControl {
    /// Put UI chip into bootloader mode (boot0=1, boot1=0)
    pub async fn bootloader_mode(&mut self) {
        self.boot0.set_high();
        self.boot1.set_low();

        // Power cycle
        self.power_cycle().await;
    }

    /// Put UI chip into normal mode (boot0=0, boot1=1)
    pub async fn normal_mode(&mut self) {
        self.boot0.set_low();
        self.boot1.set_high();

        // Power cycle
        self.power_cycle().await;
    }

    /// Hold UI chip in reset
    pub async fn hold_in_reset(&mut self) {
        self.boot0.set_low();
        self.boot1.set_high();

        self.nrst.set_as_output(Speed::Low);
        self.nrst.set_low();
    }

    /// Power cycle the UI chip
    pub async fn power_cycle(&mut self) {
        // Set nrst as output
        self.nrst.set_as_output(Speed::Low);
        self.nrst.set_low();
        Timer::after(Duration::from_millis(10)).await;

        self.nrst.set_high();
        Timer::after(Duration::from_millis(10)).await;

        self.nrst.set_low();
        Timer::after(Duration::from_millis(10)).await;

        // Switch to input mode with pull-up (matching C code behavior)
        self.nrst.set_as_input(Pull::Up);
    }
}
