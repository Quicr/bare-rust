//! # Keyboard Module
//!
//! This module provides functionality for interacting and retrieving key presses.
//!
//!
//! The keyboard is interfaced using a matrix of rows and columns.
//!
//! - **Keyboard Columns**: These are configured as output pins with a default state of low (pulled low). There are 5 columns.
//! - **Keyboard Rows**: These are configured as input pins with a default state of low (pulled down). There are 7 rows.
//!
//! ## Functions
//!
//! - `new`: Creates a new instance of the `Keyboard` struct.
//! - `init`: Initializes the keyboard interface.
//! - `get_key`: Retrieves the key that has been pressed.
//!

extern crate hal;

#[cfg(feature = "std")]
extern crate std;

pub struct Keyboard {}

impl crate::keyboard::Keyboard {
    #[inline(never)]
    pub fn new() -> Self {
        crate::keyboard::Keyboard {}
    }

    #[inline(never)]
    pub fn init(&self) {}

    /// Gets the key that has been pressed. This will return just one key at a time.
    ///
    /// # Returns
    /// A `u8` representing the key pressed. Returns 0 if no key has been pressed.

    pub fn get_key(&self) -> u8 {
        // returns 0 if no key has been pressed
        0
    }
}

const Q10_COLS: usize = 5;
const Q10_ROWS: usize = 7;

const SYM: u8 = 1; // Replace with the actual byte value
const ALT: u8 = 2; // Replace with the actual byte value
const MIC: u8 = 3; // Replace with the actual byte value
const SHF: u8 = 4; // Replace with the actual byte value
const DLR: u8 = 5; // Replace with the actual byte value
const SPK: u8 = 6; // Replace with the actual byte value
const BAK: u8 = 0x08; // Replace with the actual byte value
const ENT: u8 = 0x0D; // Replace with the actual byte value
const SPC: u8 = b' '; // 32

const BASE_CHAR_MAP: [[u8; Q10_ROWS]; Q10_COLS] = [
    [b'Q', b'W', SYM, b'A', ALT, SPC, MIC],    // col1
    [b'E', b'S', b'D', b'P', b'X', b'Z', SHF], // col2
    [b'R', b'G', b'T', SHF, b'V', b'C', b'F'], // col3
    [b'U', b'H', b'Y', ENT, b'B', b'N', b'J'], // col4
    [b'O', b'L', b'I', BAK, DLR, b'M', b'K'],  // col5
];

const SYMB_CHAR_MAP: [[u8; Q10_ROWS]; Q10_COLS] = [
    [b'#', b'1', SYM, b'*', ALT, SPC, b'0'],   // col1
    [b'2', b'4', b'5', b'@', b'8', b'7', SHF], // col2
    [b'3', b'/', b'(', SHF, b'?', b'9', b'6'], // col3
    [b'_', b':', b')', ENT, b'!', b',', b';'], // col4
    [b'+', b'"', b'-', BAK, SPK, b'.', b'\''], // col5
];

pub fn remove_me() {
    // TODO - remove this functions
    let _ = (
        BASE_CHAR_MAP,
        SYMB_CHAR_MAP,
        Q10_COLS,
        Q10_ROWS,
        SYM,
        ALT,
        MIC,
        SHF,
        DLR,
        SPK,
        BAK,
        ENT,
        SPC,
    );
}
