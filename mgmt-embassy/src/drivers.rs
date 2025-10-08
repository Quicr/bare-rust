use embassy_stm32::{
    gpio::{Flex, Level, Output, Pull, Speed},
    Peri,
};
use embassy_time::Delay;
use embedded_hal::delay::DelayNs;

/// RGB LED controller
pub struct RgbLed {
    r: Output<'static>,
    g: Output<'static>,
    b: Output<'static>,
}

impl RgbLed {
    pub fn new(
        r: Peri<'static, impl embassy_stm32::gpio::Pin>,
        g: Peri<'static, impl embassy_stm32::gpio::Pin>,
        b: Peri<'static, impl embassy_stm32::gpio::Pin>,
    ) -> Self {
        Self {
            r: Output::new(r, Level::High, Speed::Low),
            g: Output::new(g, Level::High, Speed::Low),
            b: Output::new(b, Level::High, Speed::Low),
        }
    }

    pub fn set_rgb(&mut self, r: bool, g: bool, b: bool) {
        // LEDs are active low
        self.r.set_level(if r { Level::Low } else { Level::High });
        self.g.set_level(if g { Level::Low } else { Level::High });
        self.b.set_level(if b { Level::Low } else { Level::High });
    }

    #[allow(dead_code)]
    pub fn off(&mut self) {
        self.set_rgb(false, false, false);
    }

    #[allow(dead_code)]
    pub fn set_red(&mut self) {
        self.set_rgb(true, false, false);
    }

    #[allow(dead_code)]
    pub fn set_green(&mut self) {
        self.set_rgb(false, true, false);
    }

    #[allow(dead_code)]
    pub fn set_blue(&mut self) {
        self.set_rgb(false, false, true);
    }

    #[allow(dead_code)]
    pub fn toggle_red(&mut self) {
        self.r.toggle();
    }

    #[allow(dead_code)]
    pub fn toggle_green(&mut self) {
        self.g.toggle();
    }

    #[allow(dead_code)]
    pub fn toggle_blue(&mut self) {
        self.b.toggle();
    }
}

/// Control pins for UI chip
pub struct UiControl {
    pub nrst: Flex<'static>,
    pub boot0: Output<'static>,
    pub boot1: Output<'static>,
}

impl UiControl {
    pub fn new(
        nrst: Peri<'static, impl embassy_stm32::gpio::Pin>,
        boot0: Peri<'static, impl embassy_stm32::gpio::Pin>,
        boot1: Peri<'static, impl embassy_stm32::gpio::Pin>,
    ) -> Self {
        let mut nrst_flex = Flex::new(nrst);
        nrst_flex.set_as_output(Speed::Low);
        nrst_flex.set_high();

        Self {
            nrst: nrst_flex,
            boot0: Output::new(boot0, Level::Low, Speed::Low),
            boot1: Output::new(boot1, Level::High, Speed::Low),
        }
    }
}

/// Control pins for NET chip
pub struct NetControl {
    pub nrst: Output<'static>,
    pub boot: Output<'static>,
}

impl NetControl {
    pub fn new(
        nrst: Peri<'static, impl embassy_stm32::gpio::Pin>,
        boot: Peri<'static, impl embassy_stm32::gpio::Pin>,
    ) -> Self {
        Self {
            nrst: Output::new(nrst, Level::High, Speed::Low),
            boot: Output::new(boot, Level::High, Speed::Low),
        }
    }
}

/// Control functions for NET chip
impl NetControl {
    /// Power cycle the NET chip reset pin
    fn power_cycle(&mut self, delay_ms: u64) {
        let mut delay = Delay;
        self.nrst.set_low();
        delay.delay_ms(delay_ms as u32);
        self.nrst.set_high();
    }

    /// Put NET chip into bootloader mode
    pub fn bootloader_mode(&mut self) {
        self.power_cycle(10);

        // Bring boot low for ESP bootloader mode
        self.boot.set_low();

        // Power cycle
        self.power_cycle(10);
    }

    /// Put NET chip into normal mode
    pub fn normal_mode(&mut self) {
        self.boot.set_high();

        // Power cycle
        self.power_cycle(10);
    }

    /// Hold NET chip in reset
    pub fn hold_in_reset(&mut self) {
        let mut delay = Delay;
        self.boot.set_high();

        // Reset and hold
        self.nrst.set_low();
        delay.delay_ms(100);
    }
}

/// Control functions for UI chip
impl UiControl {
    /// Put UI chip into bootloader mode (boot0=1, boot1=0)
    pub fn bootloader_mode(&mut self) {
        self.boot0.set_high();
        self.boot1.set_low();

        // Power cycle
        self.power_cycle();
    }

    /// Put UI chip into normal mode (boot0=0, boot1=1)
    pub fn normal_mode(&mut self) {
        self.boot0.set_low();
        self.boot1.set_high();

        // Power cycle
        self.power_cycle();
    }

    /// Hold UI chip in reset
    pub fn hold_in_reset(&mut self) {
        self.boot0.set_low();
        self.boot1.set_high();

        self.nrst.set_as_output(Speed::Low);
        self.nrst.set_low();
    }

    /// Power cycle the UI chip
    pub fn power_cycle(&mut self) {
        let mut delay = Delay;
        // Set nrst as output
        self.nrst.set_as_output(Speed::Low);
        self.nrst.set_low();
        delay.delay_ms(10);

        self.nrst.set_high();
        delay.delay_ms(10);

        self.nrst.set_low();
        delay.delay_ms(10);

        // Switch to input mode with pull-up (matching C code behavior)
        self.nrst.set_as_input(Pull::Up);
    }
}
