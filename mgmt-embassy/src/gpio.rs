use embassy_stm32::{
    gpio::{Level, Output, Speed},
    Peri,
};

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

    pub fn off(&mut self) {
        self.set_rgb(false, false, false);
    }

    pub fn set_red(&mut self) {
        self.set_rgb(true, false, false);
    }

    pub fn set_green(&mut self) {
        self.set_rgb(false, true, false);
    }

    pub fn set_blue(&mut self) {
        self.set_rgb(false, false, true);
    }

    pub fn toggle_red(&mut self) {
        self.r.toggle();
    }

    pub fn toggle_green(&mut self) {
        self.g.toggle();
    }

    pub fn toggle_blue(&mut self) {
        self.b.toggle();
    }
}

/// Control pins for UI chip
pub struct UiControl {
    pub nrst: Output<'static>,
    pub boot0: Output<'static>,
    pub boot1: Output<'static>,
}

impl UiControl {
    pub fn new(
        nrst: Peri<'static, impl embassy_stm32::gpio::Pin>,
        boot0: Peri<'static, impl embassy_stm32::gpio::Pin>,
        boot1: Peri<'static, impl embassy_stm32::gpio::Pin>,
    ) -> Self {
        Self {
            nrst: Output::new(nrst, Level::High, Speed::Low),
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

/// All GPIO peripherals
pub struct GpioPeripherals {
    pub led_a: RgbLed,
    pub led_b: RgbLed,
    pub ui_control: UiControl,
    pub net_control: NetControl,
}
