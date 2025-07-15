#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
mod tests {
    use bsp::console::Print;
    use bsp::*;

    #[test]
    fn test_init() {
        let mut bsp = BSP::new();
        bsp.init();
        bsp.validate();

        let (_state, _changed) = bsp.buttons.read_ptt();
        let (_state, _changed) = bsp.buttons.read_ptt();
        assert_eq!(_state, false);
        assert_eq!(_changed, false);

        let (_state, _changed) = bsp.buttons.read_ai();
        let (_state, _changed) = bsp.buttons.read_ai();
        assert_eq!(_state, false);
        assert_eq!(_changed, false);

        let message: &[u8] = b"test";
        message.print_console();

        let number: u64 = 123;
        number.print_console();

        bsp::debug::set(0, true);
        bsp::debug::set(0, false);

        led::set(led::Color::Green);
    }
}
