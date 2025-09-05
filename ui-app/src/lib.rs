#![no_std]

#[derive(Clone)]
pub enum Event {
    PttDown,
    PttUp,
    AiDown,
    AiUp,
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

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> usize;
}

pub trait Outputs {
    fn status_led(&mut self) -> &mut impl Led;
    fn mgmt_tx(&mut self) -> &mut impl Write;
}

#[derive(Debug)]
pub struct App {
    ptt_down: bool,
    ai_down: bool,
}

impl App {
    pub fn start(out: &mut impl Outputs) -> Self {
        out.status_led().set_color(Color::Black);

        Self {
            ptt_down: false,
            ai_down: false,
        }
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
        }

        out.status_led().set(false, self.ptt_down, self.ai_down);
    }
}
