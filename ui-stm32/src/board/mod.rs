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
use embassy_stm32::{
    exti::ExtiInput,
    gpio::Output,
    i2c::{mode::Master, I2c},
    mode::Blocking,
};
use embassy_time::Delay;
use embedded_hal::delay::DelayNs;
use ui_app::Led;

mod keyboard;
pub use keyboard::Keyboard;

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

struct Eeprom<'a> {
    i2c: &'a mut I2c<'static, Blocking, Master>,
}

impl Eeprom<'_> {
    const I2C_ADDR: u8 = 0x50;
}

impl<'a> ui_app::Eeprom for Eeprom<'a> {
    fn read(&mut self, data: &mut [u8; 256]) {
        const START_ADDR: u8 = 0;
        self.i2c
            .blocking_write_read(Self::I2C_ADDR, &[START_ADDR], data)
            .unwrap();
    }

    fn write(&mut self, data: &[u8; 256]) {
        // EEPROM allows us to write 16 bytes at a time.  The first byte in the write is the start
        // address.
        const CHUNK_SIZE: usize = 16;
        let mut write_data = [0; CHUNK_SIZE + 1];
        for start in (0_usize..0xff).step_by(16) {
            write_data[0] = start as u8;
            write_data[1..].copy_from_slice(&data[start..(start + CHUNK_SIZE)]);

            self.i2c
                .blocking_write(Self::I2C_ADDR, &write_data)
                .unwrap();

            // XXX(RLB) The chip takes some time to write.  The datasheet says there's a way to
            // poll on ACK so that you don't have to wait, but it's not clear that this can be
            // implemented through the abstractions we have.  These operations are not
            // time-sensitive, so a delay should be fine.
            Delay.delay_ns(10_000_000);
        }
    }
}
