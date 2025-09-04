use crate::hal::gpio;

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

pub struct Led<R, G, B, const INVERT: bool = false> {
    r: gpio::Output<R>,
    g: gpio::Output<G>,
    b: gpio::Output<B>,
}

impl<R, G, B, const INVERT: bool> Default for Led<R, G, B, INVERT>
where
    R: gpio::Fields,
    G: gpio::Fields,
    B: gpio::Fields,
{
    fn default() -> Self {
        let r = R::output();
        let g = G::output();
        let b = B::output();

        r.set_low();
        g.set_low();
        b.set_low();

        Self { r, g, b }
    }
}

impl<R, G, B, const INVERT: bool> Led<R, G, B, INVERT>
where
    R: gpio::Fields,
    G: gpio::Fields,
    B: gpio::Fields,
{
    pub fn set(&self, c: Color) {
        let (r, g, b) = c.rgb();
        self.r.set(r ^ INVERT);
        self.g.set(g ^ INVERT);
        self.b.set(b ^ INVERT);
    }
}
