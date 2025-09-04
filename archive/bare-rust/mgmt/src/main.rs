//! Application main loop and entry point

#![no_std]
#![no_main]

extern crate hal;

use hal::{cpu, gpio, watch_dog};

mod stack;
mod startup;

#[no_mangle]
#[inline(never)]
/// Entry point for the application when the `std` feature is not enabled.
pub extern "C" fn main() -> ! {
    my_main();

    loop {}
}

#[inline(never)]
/// Main function that initializes the system and runs the task manager.
fn my_main() {
    pub const CONSOLE_TX: gpio::Pin = gpio::Pin(cpu::GPIOA, 9);
    pub const CONSOLE_RX: gpio::Pin = gpio::Pin(cpu::GPIOA, 10);

    pub const UI_TX: gpio::Pin = gpio::Pin(cpu::GPIOA, 2);
    pub const UI_RX: gpio::Pin = gpio::Pin(cpu::GPIOA, 3);

    pub const UI_BOOT0: gpio::Pin = gpio::Pin(cpu::GPIOA, 15);
    pub const UI_NRST: gpio::Pin = gpio::Pin(cpu::GPIOB, 3);

    pub const NET_BOOT0: gpio::Pin = gpio::Pin(cpu::GPIOB, 5);
    pub const NET_NRST: gpio::Pin = gpio::Pin(cpu::GPIOB, 4);

    pub const MCLK: gpio::Pin = gpio::Pin(cpu::GPIOA, 8);

    pub const MCLK_FREQ: u32 = 24_000_000;
    pub const CLOCK_HSE_FREQ: u32 = 16_000_000;

    hal::init(CLOCK_HSE_FREQ);

    hal::uart::init1(115_200, CONSOLE_TX, CONSOLE_RX);

    hal::uart::init2(115200, UI_RX, UI_TX);

    hal::watch_dog::init();

    const LED_RED_PIN: gpio::Pin = gpio::Pin(cpu::GPIOA, 4);
    const LED_GREEN_PIN: gpio::Pin = gpio::Pin(cpu::GPIOA, 6);
    const LED_BLUE_PIN: gpio::Pin = gpio::Pin(cpu::GPIOA, 7);

    LED_GREEN_PIN.output();
    LED_RED_PIN.output();
    LED_BLUE_PIN.output();

    LED_GREEN_PIN.high();
    LED_RED_PIN.high();
    LED_BLUE_PIN.low(); // turn on blue LED

    hal::clock::configure_mco(MCLK, MCLK_FREQ);

    // make sure that boot pins are low before any reset asserted
    UI_BOOT0.output();
    NET_BOOT0.output();
    UI_BOOT0.low();
    NET_BOOT0.low();

    UI_NRST.open_drain();
    UI_NRST.pullup();
    NET_NRST.open_drain();
    NET_NRST.pullup();

    // put chips into reset
    UI_NRST.low();
    NET_NRST.low();

    {
        let str = "MGMT: Starting\r\n";
        for c in str.bytes() {
            hal::uart::write1(c);
        }
    }
    // take chips out of reset
    UI_NRST.high();
    NET_NRST.high();

    // TODO this is off in debug as it stop CPU from staying in error state
    if !cfg!(debug_assertions) {
        watch_dog::start();
    } else {
        let str = "MGMT: in DEBUG mode\r\n";
        for c in str.bytes() {
            hal::uart::write1(c);
            watch_dog::alive();
        }
    }

    let w = hal::watch_dog::is_enabled();
    if !w {
        let str = "MGMT: No Watchdog\r\n";
        for c in str.bytes() {
            hal::uart::write1(c);
            watch_dog::alive();
        }
    } else {
        let str = "MGMT: Watchdog Enabled\r\n";
        for c in str.bytes() {
            hal::uart::write1(c);
            watch_dog::alive();
        }
    }

    LED_GREEN_PIN.low(); // turn on green LED
    LED_RED_PIN.high();
    LED_BLUE_PIN.high();

    let (stack_usage, stack_current, stack_reserved) = stack::usage(false);
    let _ = (stack_usage, stack_current, stack_reserved);

    loop {
        if !hal::uart::empty2() {
            let c: u8;
            c = hal::uart::read2();
            if c != 0 {
                hal::uart::write1(c);
            }
        }
        if !hal::uart::empty1() {
            let c: u8;
            c = hal::uart::read1();
            if c != 0 {
                hal::uart::write2(c);
            }
            let local_echo = false;
            if local_echo {
                hal::uart::write1(c);
            }
        }
        watch_dog::alive();
    }
}
