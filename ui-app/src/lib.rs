#![no_std]

use bitmap_font::{tamzen::FONT_10x20, BitmapFont, TextStyle};
use core::fmt::Write;
use defmt::Format;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Dimensions,
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::Text,
};
use heapless::String;

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

pub trait Eeprom {
    fn read(&mut self, data: &mut [u8; 256]);
    fn write(&mut self, data: &[u8; 256]);
}

pub trait Outputs {
    fn status_led(&mut self) -> &mut impl Led;
    fn screen(&mut self) -> &mut impl DrawTarget<Color = Rgb565>;
    fn net_tx(&mut self) -> &mut impl NetTx;
    fn eeprom(&mut self) -> impl Eeprom;
    fn log(&mut self, message: &str);
}

#[derive(Debug)]
pub struct App {
    a_down: bool,
    b_down: bool,
    message_buffer: String<24>,
}

impl App {
    const BACKGROUND: Rgb565 = Rgb565::BLACK;
    const FONT: BitmapFont<'static> = FONT_10x20;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            a_down: false,
            b_down: false,
            message_buffer: Default::default(),
        }
    }

    pub fn start(&mut self, out: &mut impl Outputs) {
        // Extinguish the status LED
        out.status_led().set_color(Color::Black);

        // Draw a test pattern to the screen
        let rect = out.screen().bounding_box();

        rect.into_styled(PrimitiveStyle::with_fill(Self::BACKGROUND))
            .draw(out.screen())
            .unwrap_or_else(|_| panic!("graphics error"));

        let pad: u32 = 10;
        let diameter: u32 = 20;
        let width = rect.size.width;
        let height = rect.size.height;

        let mut dot = |left, top, color| {
            Circle::new(Point::new(left as i32, top as i32), diameter)
                .into_styled(PrimitiveStyle::with_fill(color))
                .draw(out.screen())
                .unwrap_or_else(|_| panic!("graphics error"));
        };

        dot(pad, pad, Rgb565::RED);
        dot(width - pad - diameter, pad, Rgb565::GREEN);
        dot(pad, height - pad - diameter, Rgb565::BLUE);
        dot(
            width - pad - diameter,
            height - pad - diameter,
            Rgb565::YELLOW,
        );

        let text = Text::new(
            "Hello World!",
            Point { x: 10, y: 30 },
            TextStyle::new(&Self::FONT, BinaryColor::On),
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

            Event::KeyDown(key, value) => {
                // Log the key press
                let mut msg: String<32> = Default::default();
                write!(&mut msg, "key down: {:?} {:?}", key, value).unwrap();
                out.log(&msg);

                // If this key press is a return, then clear things out
                if let Key::Enter = key {
                    let mut msg: String<64> = Default::default();
                    write!(&mut msg, "sending message: {}", self.message_buffer).unwrap();
                    out.log(&msg);

                    self.message_buffer.clear();

                    let width = out.screen().bounding_box().size.width;
                    let height = Self::FONT.height();
                    Rectangle::new(Point::new(0, 0), Size::new(width, height))
                        .into_styled(PrimitiveStyle::with_fill(Self::BACKGROUND))
                        .draw(out.screen())
                        .unwrap_or_else(|_| panic!("graphics error"));
                    return;
                }

                // Otherwise, if it's a character, add it to the buffer and render it to the screen
                let KeyValue::Char(c) = value else {
                    return;
                };

                if self.message_buffer.len() == self.message_buffer.capacity() {
                    // Ignore any characters beyond the capacity of the message buffer
                    return;
                }

                self.message_buffer.push(c).unwrap();

                let text = Text::new(
                    &self.message_buffer,
                    Point { x: 0, y: 0 },
                    TextStyle::new(&Self::FONT, BinaryColor::On),
                );

                let mut binary_display =
                    BinaryDisplay::new(Rgb565::WHITE, Rgb565::BLACK, out.screen());
                text.draw(&mut binary_display)
                    .unwrap_or_else(|_| panic!("graphics error"));
            }

            Event::KeyUp(key) => {
                let mut msg: heapless::String<32> = Default::default();
                write!(&mut msg, "key up: {:?}", key).unwrap();
                out.log(&msg);
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
