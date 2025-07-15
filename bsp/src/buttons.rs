//! # Buttons Module
//!
//! This module provides functionality for reading if the buttons on the board are pressed.
//!
//! ## Functions
//!
//! - `read_ptt`: Reads the state of the PTT (Push-To-Talk) button.
//! - `read_ai`: Reads the state of the AI button.
//!

extern crate hal;

use crate::board;

pub struct Buttons {
    prev_ptt: bool,
    prev_ai: bool,
}

impl Buttons {
    #[inline(never)]
    pub fn new() -> Self {
        Buttons {
            prev_ptt: false,
            prev_ai: false,
        }
    }

    #[inline(never)]
    pub fn init(&mut self) {
        if board::info::HAS_PTT_BUTTON {
            board::info::PTT_BUTTON.input();

            if board::info::PTT_BUTTON_PULL_UP {
                board::info::PTT_BUTTON.pullup();
            } else {
                board::info::PTT_BUTTON.pulldown();
            }
        }

        if board::info::HAS_AI_BUTTON {
            board::info::AI_BUTTON.input();

            if board::info::AI_BUTTON_PULL_UP {
                board::info::AI_BUTTON.pullup();
            } else {
                board::info::AI_BUTTON.pulldown();
            }
        }
    }

    #[inline(never)]
    pub fn validate(&self) {}

    /// Reads the state of the PTT (Push-To-Talk) button.
    ///
    /// This function checks the current state of the PTT button and compares it with the state when it was previously called.
    /// It returns a tuple containing the current state and a boolean indicating whether the state has changed.
    ///
    /// # Returns
    ///
    /// * `(bool, bool)` - A tuple where the first element is the current state of the PTT button
    ///   (`true` if pressed, `false` otherwise), and the second element  if the state has changed
    ///   since the last read.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bsp::BSP;
    /// use bsp::buttons;
    ///  let mut bsp = BSP::new();
    ///  bsp.init();
    ///
    /// let (state, changed) = bsp.buttons.read_ptt();
    /// if changed {
    ///     if state {
    ///         println!("PTT button pressed");
    ///     } else {
    ///         println!("PTT button released");
    ///     }
    /// }
    /// ```
    ///
    ///
    pub fn read_ptt(&mut self) -> (bool, bool) {
        if board::info::HAS_PTT_BUTTON {
            let state = board::info::PTT_BUTTON.read() != board::info::PTT_BUTTON_PULL_UP;
            let changed = state != self.prev_ptt;
            self.prev_ptt = state;
            return (state, changed);
        }
        (false, false)
    }

    /// Reads the state of the AI  button.
    ///
    /// This function checks the current state of the AI button and compares it with the state when it was previously called.
    /// It returns a tuple containing the current state and a boolean indicating whether the state has changed.
    ///
    /// # Returns
    ///
    /// * `(bool, bool)` - A tuple where the first element is the current state of the AI button
    ///   (`true` if pressed, `false` otherwise), and the second element if the state has changed
    ///   since the last read.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bsp::BSP;
    /// use bsp::buttons;
    ///
    /// let mut bsp = BSP::new();
    /// bsp.init();
    ///
    /// let (state, changed) = bsp.buttons.read_ai();
    /// if changed {
    ///     if state {
    ///         println!("AI button pressed");
    ///     } else {
    ///         println!("AI button released");
    ///     }
    /// }
    /// ```
    ///
    pub fn read_ai(&mut self) -> (bool, bool) {
        if board::info::HAS_AI_BUTTON {
            let state = board::info::AI_BUTTON.read() != board::info::AI_BUTTON_PULL_UP;
            let changed = state != self.prev_ai;
            self.prev_ai = state;
            return (state, changed);
        }
        (false, false)
    }
}
