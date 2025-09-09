#![no_std]

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
    PttDown,
    PttUp,
    AiDown,
    AiUp,
    KeyDown(Key, KeyValue),
    KeyUp(Key),
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

fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    // Discard the low-order bits of the colors
    (((r as u16) >> 3) << 11) | (((g as u16) >> 2) << 5) | ((b as u16) >> 3)
}

pub trait Screen {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn fill(&mut self, color: u16);
    fn draw(&mut self, left: usize, right: usize, top: usize, bottom: usize, data: &[u16]);
}

pub trait Outputs {
    fn status_led(&mut self) -> &mut impl Led;
    fn screen(&mut self) -> &mut impl Screen;
    fn log(&mut self, message: &str);
}

#[derive(Debug)]
pub struct App {
    ptt_down: bool,
    ai_down: bool,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            ptt_down: false,
            ai_down: false,
        }
    }

    pub fn start(&mut self, out: &mut impl Outputs) {
        // Extinguish the status LED
        out.status_led().set_color(Color::Black);

        // Draw a test pattern to the screen
        const SIZE: usize = 10;
        let screen = out.screen();
        let mut data = [0_u16; SIZE * SIZE];

        // Background
        screen.fill(rgb565(0x88, 0x88, 0x88));

        // Upper left = R
        let (x0, x1, y0, y1) = (SIZE, SIZE + SIZE, SIZE, SIZE + SIZE);
        data.fill(rgb565(0xFF, 0x00, 0x00));
        screen.draw(x0, x1, y0, y1, &data);

        // Upper right = G
        let (x0, x1, y0, y1) = (
            screen.width() - SIZE - SIZE,
            screen.width() - SIZE,
            SIZE,
            SIZE + SIZE,
        );
        data.fill(rgb565(0x00, 0xFF, 0x00));
        screen.draw(x0, x1, y0, y1, &data);

        // Lower left = B
        let (x0, x1, y0, y1) = (
            SIZE,
            SIZE + SIZE,
            screen.height() - SIZE - SIZE,
            screen.height() - SIZE,
        );
        data.fill(rgb565(0x00, 0x00, 0xFF));
        screen.draw(x0, x1, y0, y1, &data);

        // Lower right = Y
        let (x0, x1, y0, y1) = (
            screen.width() - SIZE - SIZE,
            screen.width() - SIZE,
            screen.height() - SIZE - SIZE,
            screen.height() - SIZE,
        );
        data.fill(rgb565(0xFF, 0xFF, 0x00));
        screen.draw(x0, x1, y0, y1, &data);
    }

    pub fn handle(&mut self, event: Event, out: &mut impl Outputs) {
        match event {
            Event::PttDown => {
                self.ptt_down = true;
            }
            Event::PttUp => {
                self.ptt_down = false;
            }
            Event::AiDown => {
                self.ai_down = true;
            }
            Event::AiUp => {
                self.ai_down = false;
            }
            Event::KeyDown(_key, _value) => {
                out.log("key down");
            }
            Event::KeyUp(_key) => {
                out.log("key up");
            }
        }

        out.status_led().set(false, self.ptt_down, self.ai_down);
    }
}
