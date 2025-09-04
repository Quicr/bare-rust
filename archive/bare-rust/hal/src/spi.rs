#[allow(unused_imports)]
use core::ptr;

#[allow(unused_imports)]
use super::cpu;
pub use super::cpu::*;
use super::gpio;

#[cfg(feature = "stm32f072")]
#[inline(never)]
pub fn init1(spi_freq: u32, scl_pin: gpio::Pin, sda_pin: gpio::Pin) {
    let _ = (spi_freq, scl_pin, sda_pin);
    assert!(false);
}

#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn init1(spi_freq: u32, scl_pin: gpio::Pin, sda_pin: gpio::Pin) {
    scl_pin.output();
    sda_pin.output();

    sda_pin.pullup();
    scl_pin.pulldown();

    // enable clock for SPI1
    cpu::write!(RCC.apb2enr[SPI1EN;1], 0b1);

    // set up pins - AF from Figur 26 of RM0090 Reference Manual
    // also see table 9 "Alternate function mapping" table of the datasheet stm32f405zg.pdf
    scl_pin.alt_fun(5, true); // AF5 for PA5 is SPI1_SCK
    sda_pin.alt_fun(5, true); // AF5 for PA7 is SPI1_MOSI
    assert!(scl_pin.0 == gpio::GPIOA);
    assert!(sda_pin.0 == gpio::GPIOA);
    assert!(scl_pin.1 == 5);
    assert!(sda_pin.1 == 7);

    // set up the SPI
    cpu::write!(SPI1.cr1[DFF;1], 0b0); // set to 8 bit frame
    cpu::write!(SPI1.cr1[LSBFIRST;1], 0b0); // set MSB send first
    cpu::write!(SPI1.cr1[RXONLY;1], 0b0); // set to full duplex
    cpu::write!(SPI1.cr1[CRCEN;1], 0b0); // disable CRC

    // TODO - Brett has SPI at about 2.5 Mhz not 10 MHz
    assert!(spi_freq >= 48_000_000 / 8);
    //cpu::write!( SPI1.cr1[BR;3] , 0b100 ); // set baud rate to 2.5 Mhz (0b100 )
    cpu::write!( SPI1.cr1[BR;3] , 0b010 ); // set baud rate to 10 Mhz (0b010 )

    cpu::write!( SPI1.cr1[MSTR;1] , 0b1 ); // set to master mode
    cpu::write!(SPI1.cr1[BIDIMODE;1], 0b1); // set BIDIMODE to 1 line both directions
    cpu::write!(SPI1.cr1[BIDIOE;1], 0b1); // set BIDIOE to output

    cpu::write!( SPI1.cr1[SSM;1] , 0b1 );
    cpu::write!( SPI1.cr1[SSI;1] , 0b1 );

    cpu::write!( SPI1.cr1[CPOL;1] , 0b0 ); // low when idle
    cpu::write!( SPI1.cr1[CPHA;1] , 0b0 ); // sample on rising edge

    cpu::write!( SPI1.cr1[SPE;1] , 0b0 ); // do ot enable SPI yet
    cpu::write!( SPI1.cr1[SPE;1] , 0b1 ); // enable SPI
}

#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn write1(data: &[u8]) {
    // wait for SPI to not be busy
    while cpu::read!(SPI1.sr[BSY;1]) != 0 {}

    for &d in data {
        cpu::write!(SPI1.dr, d as u32); // send 8 buts of data

        // wait for transmit buffer to be empty
        while cpu::read!(SPI1.sr[TXE;1]) == 0 {}
    }

    // wait for SPI to not be busy
    while cpu::read!(SPI1.sr[BSY;1]) != 0 {}
}

#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn write1_wide(data: &[u16]) {
    // wait for SPI to not be busy
    while cpu::read!(SPI1.sr[BSY;1]) != 0 {}

    cpu::write!(SPI1.cr1[DFF;1], 0b1); // set to 16 bit frame

    for &d in data {
        cpu::write!(SPI1.dr, d as u32); // send 16 bits of data

        // wait for transmit buffer to be empty
        while cpu::read!(SPI1.sr[TXE;1]) == 0 {}
    }

    // wait for SPI to not be busy
    while cpu::read!(SPI1.sr[BSY;1]) != 0 {}

    cpu::write!(SPI1.cr1[DFF;1], 0b0); // set to 8 bit frame
}
