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
        todo!() // Copy keymap
    }

    fn sym(&self) -> KeyValue {
        todo!() // Copy keymap
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
    fn new(cols: [Output<'static>; COLS], rows: [Input<'static>; ROWS]) -> Self {
        Self {
            cols,
            rows,
            pressed: [[false; ROWS]; COLS],
        }
    }

    fn scan(&mut self) -> Vec<Event, { COLS * ROWS }> {
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
                    events.push(Event::KeyDown(key, key.value(sym, shift)));
                    self.pressed[j][i] = pressed;
                }

                if !pressed && self.pressed[j][i] {
                    events.push(Event::KeyUp(key));
                    self.pressed[j][i] = pressed;
                }
            }
        }

        events
    }
}

const KEY_MAP: [[Key; ROWS]; COLS] = todo!();
