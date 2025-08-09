use core::fmt::Write;

pub enum Event {
    ConsoleRx(u8),
    Timer1Hz,
}

pub trait Toggle {
    fn toggle(&mut self);
}

#[derive(Default)]
pub struct App {
    line: heapless::String<128>,
}

impl App {
    pub fn timer_1hz(&mut self, led: &mut dyn Toggle) {
        led.toggle();
    }

    pub fn handle_byte(&mut self, byte: u8, tx: &mut dyn Write) {
        // Still building string
        if byte != b'\r' && self.line.len() < self.line.capacity() {
            self.line.push(byte as char).ok();
            return;
        }

        // End-of-line => send response
        tx.write_str("hello: ").ok();
        tx.write_str(self.line.as_str()).ok();
        tx.write_str("\r\n").ok();

        self.line.clear();
    }

    pub fn handle(&mut self, event: Event, tx: &mut dyn Write, led_a: &mut dyn Toggle) {
        match event {
            Event::ConsoleRx(b) => self.handle_byte(b, tx),
            Event::Timer1Hz => self.timer_1hz(led_a),
        }
    }
}
