// Negotiate board selection using features
#[cfg(not(any(feature = "ev13", feature = "ev12")))]
compile_error!("Please select board (ev12 or ev13)");

#[cfg(all(feature = "ev12", not(feature = "ev13")))]
mod ev12;

#[cfg(all(feature = "ev13", not(feature = "ev12")))]
mod ev13;

#[cfg(all(feature = "ev13", feature = "ev12"))]
compile_error!("Please select only one board");

// Expose the selected board
#[allow(unused_imports)]
#[cfg(feature = "ev12")]
pub use ev12::*;

#[allow(unused_imports)]
#[cfg(feature = "ev13")]
pub use ev13::*;

// Provide some common functionality
use embassy_stm32::{exti::ExtiInput, gpio::Output};
use ui_app::Led;

mod keyboard;
pub use keyboard::Keyboard;

mod screen;
pub use screen::Screen;

mod net;
pub use net::{NetRx, NetTx};

mod audio;
pub use audio::AudioControl;

struct StatusLed {
    r: Output<'static>,
    g: Output<'static>,
    b: Output<'static>,
}

impl Led for StatusLed {
    fn set(&mut self, r: bool, g: bool, b: bool) {
        self.r.set_level((!r).into());
        self.g.set_level((!g).into());
        self.b.set_level((!b).into());
    }
}

pub type Button = ExtiInput<'static>;
