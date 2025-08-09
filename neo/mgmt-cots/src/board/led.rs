use crate::app::Toggle;
use stm32f0xx_hal::prelude::_embedded_hal_gpio_OutputPin as OutputPin;

#[allow(dead_code)] // Don't warn if we don't use every color
#[derive(Copy, Clone)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Blue,
    Teal,
    Yellow,
    Purple,
}

impl Color {
    fn rgb(&self) -> (bool, bool, bool) {
        match self {
            Self::Black => (false, false, false),
            Self::Red => (true, false, false),
            Self::Green => (false, true, false),
            Self::Blue => (false, false, true),
            Self::Teal => (false, true, true),
            Self::Purple => (true, false, true),
            Self::Yellow => (true, true, false),
            Self::White => (true, true, true),
        }
    }
}

trait Set {
    fn set(&mut self, high: bool);
}

impl<T: OutputPin> Set for T {
    fn set(&mut self, high: bool) {
        let _ = if high {
            self.set_high()
        } else {
            self.set_low()
        };
    }
}

pub struct Led<R, G, B, const INVERT: bool = false> {
    on: bool,
    color: Color,
    r: R,
    g: G,
    b: B,
}

impl<R, G, B, const INVERT: bool> Led<R, G, B, INVERT>
where
    R: OutputPin,
    G: OutputPin,
    B: OutputPin,
{
    pub fn new(r: R, g: G, b: B) -> Self {
        Self {
            on: false,
            color: Color::Black,
            r,
            g,
            b,
        }
    }

    fn raw_set(&mut self, c: Color) {
        let (r, g, b) = c.rgb();
        self.r.set(r ^ INVERT);
        self.g.set(g ^ INVERT);
        self.b.set(b ^ INVERT);
    }

    pub fn set(&mut self, c: Color) {
        self.on = true;
        self.color = c;
        self.raw_set(c);
    }
}

impl<R, G, B, const INVERT: bool> Toggle for Led<R, G, B, INVERT>
where
    R: OutputPin,
    G: OutputPin,
    B: OutputPin,
{
    fn toggle(&mut self) {
        self.on = !self.on;
        if self.on {
            self.raw_set(self.color);
        } else {
            self.raw_set(Color::Black);
        }
    }
}
