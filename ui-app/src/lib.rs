#![no_std]

use bitmap_font::{tamzen::FONT_14x26, TextStyle};
use defmt::Format;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Dimensions,
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum Key {
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Backspace,
    Alt,
    Dollar,
    Enter,
    LeftShift,
    Mic,
    Space,
    Sym,
    RightShift,
}

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum KeyValue {
    Char(char),
    Backspace,
    Alt,
    Speaker,
    Enter,
    LeftShift,
    Mic,
    Space,
    Sym,
    RightShift,
}

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum Button {
    A,
    B,
}

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum FromNet {
    Pong,
}

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum ToNet {
    Ping,
}

#[derive(Copy, Clone, Debug, PartialEq, Format)]
pub enum Event {
    ButtonDown(Button),
    ButtonUp(Button),
    KeyDown(Key, KeyValue),
    KeyUp(Key),
    FromNet(FromNet),
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Blue,
    Cyan,
    Purple,
    Yellow,
    White,
}

impl Color {
    pub fn from(r: bool, g: bool, b: bool) -> Self {
        match (r, g, b) {
            (false, false, false) => Self::Black,
            (true, false, false) => Self::Red,
            (false, true, false) => Self::Green,
            (false, false, true) => Self::Blue,
            (false, true, true) => Self::Cyan,
            (true, false, true) => Self::Purple,
            (true, true, false) => Self::Yellow,
            (true, true, true) => Self::White,
        }
    }

    pub fn rgb(&self) -> (bool, bool, bool) {
        match self {
            Self::Black => (false, false, false),
            Self::Red => (true, false, false),
            Self::Green => (false, true, false),
            Self::Blue => (false, false, true),
            Self::Cyan => (false, true, true),
            Self::Purple => (true, false, true),
            Self::Yellow => (true, true, false),
            Self::White => (true, true, true),
        }
    }
}

pub trait Led {
    fn set(&mut self, r: bool, g: bool, b: bool);

    fn set_color(&mut self, color: Color) {
        let (r, g, b) = color.rgb();
        self.set(r, g, b);
    }
}

struct BinaryDisplay<'a, C, D> {
    foreground: C,
    background: C,
    display: &'a mut D,
}

impl<'a, C, D> BinaryDisplay<'a, C, D> {
    fn new(foreground: C, background: C, display: &'a mut D) -> Self {
        Self {
            foreground,
            background,
            display,
        }
    }
}

impl<'a, C, D> Dimensions for BinaryDisplay<'a, C, D>
where
    D: Dimensions,
{
    fn bounding_box(&self) -> Rectangle {
        self.display.bounding_box()
    }
}

impl<'a, C, D> DrawTarget for BinaryDisplay<'a, C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    type Color = BinaryColor;
    type Error = D::Error;

    // Required method
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let rgb = pixels.into_iter().map(|Pixel(point, color)| match color {
            BinaryColor::On => Pixel(point, self.foreground),
            BinaryColor::Off => Pixel(point, self.background),
        });
        self.display.draw_iter(rgb)
    }
}

pub trait NetTx {
    fn write(&mut self, to_net: &ToNet);
}

pub trait Outputs {
    fn status_led(&mut self) -> &mut impl Led;
    fn screen(&mut self) -> &mut impl DrawTarget<Color = Rgb565>;
    fn net_tx(&mut self) -> &mut impl NetTx;
    fn log(&mut self, message: &str);
}

#[derive(Debug)]
pub struct App {
    a_down: bool,
    b_down: bool,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            a_down: false,
            b_down: false,
        }
    }

    pub fn start(&mut self, out: &mut impl Outputs) {
        // Extinguish the status LED
        out.status_led().set_color(Color::Black);

        // Draw a test pattern to the screen
        let rect = out.screen().bounding_box();

        rect.into_styled(PrimitiveStyle::with_fill(Rgb565::new(0x88, 0x88, 0x88)))
            .draw(out.screen())
            .unwrap_or_else(|_| panic!("graphics error"));

        let mut dot = |left, top, color| {
            Circle::new(Point::new(left, top), 20)
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(out.screen())
                .unwrap_or_else(|_| panic!("graphics error"));
        };

        dot(10, 10, Rgb565::RED);
        dot(210, 10, Rgb565::GREEN);
        dot(10, 290, Rgb565::BLUE);
        dot(210, 290, Rgb565::YELLOW);

        let text = Text::new(
            "Hello World!",
            Point { x: 10, y: 30 },
            TextStyle::new(&FONT_14x26, BinaryColor::On),
        );
        let mut binary_display = BinaryDisplay::new(Rgb565::WHITE, Rgb565::BLACK, out.screen());
        text.draw(&mut binary_display)
            .unwrap_or_else(|_| panic!("graphics error"));
    }

    pub fn handle(&mut self, event: Event, out: &mut impl Outputs) {
        match event {
            Event::ButtonDown(button) => match button {
                Button::A => {
                    out.log("button a down");
                    self.a_down = true;
                    out.net_tx().write(&ToNet::Ping);
                }
                Button::B => {
                    out.log("button b down");
                    self.b_down = true;
                }
            },

            Event::ButtonUp(button) => match button {
                Button::A => {
                    out.log("button a up");
                    self.a_down = false;
                }
                Button::B => {
                    out.log("button b up");
                    self.b_down = false;
                }
            },

            Event::KeyDown(_key, _value) => {
                out.log("key down");
            }

            Event::KeyUp(_key) => {
                out.log("key up");
            }

            Event::FromNet(from_net) => match from_net {
                FromNet::Pong => {
                    out.log("pong");
                }
            },
        }

        out.status_led().set(false, self.a_down, self.b_down);
    }
}
