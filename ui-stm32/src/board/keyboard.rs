use core::array;
use embassy_stm32::gpio::{Input, Output};
use heapless::Vec;
use ui_app::{Event, Key, KeyValue};

trait Value {
    fn value(&self, sym: bool, shift: bool) -> KeyValue {
        match (sym, shift) {
            (false, false) => self.default(),
            (false, true) => self.default().shift(),
            (true, false) => self.sym(),
            (true, true) => self.sym().shift(),
        }
    }

    fn default(&self) -> KeyValue;
    fn sym(&self) -> KeyValue;
}

impl Value for Key {
    fn default(&self) -> KeyValue {
        use Key::*;
        match self {
            Q => KeyValue::Char('q'),
            W => KeyValue::Char('w'),
            E => KeyValue::Char('e'),
            R => KeyValue::Char('r'),
            T => KeyValue::Char('t'),
            Y => KeyValue::Char('y'),
            U => KeyValue::Char('u'),
            I => KeyValue::Char('i'),
            O => KeyValue::Char('o'),
            P => KeyValue::Char('p'),
            A => KeyValue::Char('a'),
            S => KeyValue::Char('s'),
            D => KeyValue::Char('d'),
            F => KeyValue::Char('f'),
            G => KeyValue::Char('g'),
            H => KeyValue::Char('h'),
            J => KeyValue::Char('j'),
            K => KeyValue::Char('k'),
            L => KeyValue::Char('l'),
            Z => KeyValue::Char('z'),
            X => KeyValue::Char('x'),
            C => KeyValue::Char('c'),
            V => KeyValue::Char('v'),
            B => KeyValue::Char('b'),
            N => KeyValue::Char('n'),
            M => KeyValue::Char('m'),
            Backspace => KeyValue::Backspace,
            Alt => KeyValue::Alt,
            Dollar => KeyValue::Char('$'),
            Enter => KeyValue::Enter,
            LeftShift => KeyValue::LeftShift,
            Mic => KeyValue::Mic,
            Space => KeyValue::Space,
            Sym => KeyValue::Sym,
            RightShift => KeyValue::RightShift,
        }
    }

    fn sym(&self) -> KeyValue {
        use Key::*;
        match self {
            Q => KeyValue::Char('#'),
            W => KeyValue::Char('1'),
            E => KeyValue::Char('2'),
            R => KeyValue::Char('3'),
            T => KeyValue::Char('('),
            Y => KeyValue::Char(')'),
            U => KeyValue::Char('_'),
            I => KeyValue::Char('-'),
            O => KeyValue::Char('+'),
            P => KeyValue::Char('@'),
            A => KeyValue::Char('*'),
            S => KeyValue::Char('4'),
            D => KeyValue::Char('5'),
            F => KeyValue::Char('6'),
            G => KeyValue::Char('/'),
            H => KeyValue::Char(':'),
            J => KeyValue::Char(';'),
            K => KeyValue::Char('\''),
            L => KeyValue::Char('"'),
            Z => KeyValue::Char('7'),
            X => KeyValue::Char('8'),
            C => KeyValue::Char('9'),
            V => KeyValue::Char('?'),
            B => KeyValue::Char('!'),
            N => KeyValue::Char(','),
            M => KeyValue::Char('.'),
            Backspace => KeyValue::Backspace,
            Alt => KeyValue::Alt,
            Dollar => KeyValue::Speaker,
            Enter => KeyValue::Enter,
            LeftShift => KeyValue::LeftShift,
            Mic => KeyValue::Char('0'),
            Space => KeyValue::Space,
            Sym => KeyValue::Sym,
            RightShift => KeyValue::RightShift,
        }
    }
}

trait Shift {
    fn shift(self) -> Self;
}

impl Shift for KeyValue {
    fn shift(self) -> Self {
        match self {
            Self::Char(c) => Self::Char(c.to_ascii_uppercase()),
            other => other,
        }
    }
}

const COLS: usize = 5;
const ROWS: usize = 7;

pub struct Keyboard {
    cols: [Output<'static>; COLS],
    rows: [Input<'static>; ROWS],
    pressed: [[bool; ROWS]; COLS],
}

// TODO(RLB) Debounce
// TODO(RLB) Caps lock
impl Keyboard {
    pub fn new(cols: [Output<'static>; COLS], rows: [Input<'static>; ROWS]) -> Self {
        Self {
            cols,
            rows,
            pressed: [[false; ROWS]; COLS],
        }
    }

    pub fn scan(&mut self) -> Vec<Event, { COLS * ROWS }> {
        const SYM_COL: usize = 0;
        const SYM_ROW: usize = 2;
        const L_SHIFT_COL: usize = 1;
        const L_SHIFT_ROW: usize = 6;
        const R_SHIFT_COL: usize = 2;
        const R_SHIFT_ROW: usize = 3;

        // Read in all pressed keys
        let pressed: [[bool; ROWS]; COLS] = array::from_fn(|j| {
            self.cols[j].set_high();
            let row = array::from_fn(|i| self.rows[i].is_high());
            self.cols[j].set_low();
            row
        });

        // Check to see if modifiers are pressed
        let sym = pressed[SYM_COL][SYM_ROW];
        let l_shift = pressed[L_SHIFT_COL][L_SHIFT_ROW];
        let r_shift = pressed[R_SHIFT_COL][R_SHIFT_ROW];
        let shift = l_shift || r_shift;

        // Translate to events
        let mut events = Vec::new();
        for (j, col) in pressed.iter().enumerate() {
            for (i, &pressed) in col.iter().enumerate() {
                let key = KEY_MAP[j][i];

                if pressed && !self.pressed[j][i] {
                    events
                        .push(Event::KeyDown(key, key.value(sym, shift)))
                        .unwrap();
                    self.pressed[j][i] = pressed;
                }

                if !pressed && self.pressed[j][i] {
                    events.push(Event::KeyUp(key)).unwrap();
                    self.pressed[j][i] = pressed;
                }
            }
        }

        events
    }
}

const KEY_MAP: [[Key; ROWS]; COLS] = [
    [
        Key::Q,
        Key::W,
        Key::Sym,
        Key::A,
        Key::Alt,
        Key::Space,
        Key::Mic,
    ],
    [
        Key::E,
        Key::S,
        Key::D,
        Key::P,
        Key::X,
        Key::Z,
        Key::LeftShift,
    ],
    [
        Key::R,
        Key::G,
        Key::T,
        Key::RightShift,
        Key::V,
        Key::C,
        Key::F,
    ],
    [Key::U, Key::H, Key::Y, Key::Enter, Key::B, Key::N, Key::J],
    [
        Key::O,
        Key::L,
        Key::I,
        Key::Backspace,
        Key::Dollar,
        Key::M,
        Key::K,
    ],
];
