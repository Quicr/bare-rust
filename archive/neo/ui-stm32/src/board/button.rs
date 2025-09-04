use crate::hal::gpio;

pub struct Button<P> {
    pin: gpio::Input<P>,
    last_state: bool,
}

impl<P> Default for Button<P>
where
    P: gpio::Fields,
{
    fn default() -> Self {
        let pin = P::input();
        pin.pull_up();
        let last_state = pin.read();
        Self { pin, last_state }
    }
}

impl<P> Button<P>
where
    P: gpio::Fields,
{
    pub fn read(&mut self) -> (bool, bool) {
        let curr_state = self.pin.read();
        let changed = curr_state == self.last_state;
        self.last_state = curr_state;
        (curr_state, changed)
    }
}
