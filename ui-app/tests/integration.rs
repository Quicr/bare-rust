use ui_app::*;

use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::Rgb565, prelude::*, primitives::Rectangle,
};

#[derive(Default)]
struct MockLed {
    color: Option<Color>,
}

impl Led for MockLed {
    fn set(&mut self, r: bool, g: bool, b: bool) {
        self.color = Some(Color::from(r, g, b));
    }
}

// TODO: Test screen logic
#[derive(Default)]
struct MockScreen;

impl Dimensions for MockScreen {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(240, 320))
    }
}

impl DrawTarget for MockScreen {
    type Color = Rgb565;
    type Error = String;

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // TODO(RLB) Store pixels
        Ok(())
    }
}

#[derive(Default)]
struct MockNetTx {
    last_to_net: Option<ToNet>,
}

impl NetTx for MockNetTx {
    fn write(&mut self, to_net: &ToNet) {
        self.last_to_net = Some(*to_net);
    }
}

#[derive(Default)]
struct MockOutputs {
    status_led: MockLed,
    screen: MockScreen,
    net_tx: MockNetTx,
    last_message: String,
}

impl Outputs for MockOutputs {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }

    fn screen(&mut self) -> &mut impl DrawTarget<Color = Rgb565> {
        &mut self.screen
    }

    fn net_tx(&mut self) -> &mut impl NetTx {
        &mut self.net_tx
    }

    fn log(&mut self, message: &str) {
        self.last_message = message.into();
    }
}

#[test]
fn default_black() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[test]
fn individual_buttons() {
    fn individual_button(button: Button, color: Color) {
        let mut outputs = MockOutputs::default();
        let mut app = App::new();
        app.start(&mut outputs);
        assert_eq!(outputs.status_led.color, Some(Color::Black));

        // Up should have no effect
        app.handle(Event::ButtonUp(button), &mut outputs);
        assert_eq!(outputs.status_led.color, Some(Color::Black));

        // Pushing the button should illuminate the LED
        app.handle(Event::ButtonDown(button), &mut outputs);
        assert_eq!(outputs.status_led.color, Some(color));

        // Down should be idempotent
        app.handle(Event::ButtonDown(button), &mut outputs);
        assert_eq!(outputs.status_led.color, Some(color));

        // Up should extinguish the LED
        app.handle(Event::ButtonUp(button), &mut outputs);
        assert_eq!(outputs.status_led.color, Some(Color::Black));

        // Up should be idempotent
        app.handle(Event::ButtonUp(button), &mut outputs);
        assert_eq!(outputs.status_led.color, Some(Color::Black));
    }

    individual_button(Button::A, Color::Green);
    individual_button(Button::B, Color::Blue);
}

#[test]
fn buttons_compose() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    app.handle(Event::ButtonDown(Button::B), &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Blue));

    app.handle(Event::ButtonDown(Button::A), &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Cyan));

    app.handle(Event::ButtonUp(Button::B), &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Green));

    app.handle(Event::ButtonUp(Button::A), &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[test]
fn key_logging() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    assert_eq!(outputs.last_message, "");

    app.handle(Event::KeyDown(Key::A, KeyValue::Char('a')), &mut outputs);
    assert_eq!(outputs.last_message, "key down: A Char('a')");

    app.handle(Event::KeyUp(Key::A), &mut outputs);
    assert_eq!(outputs.last_message, "key up: A");
}

#[test]
fn message_buffer_tolerates_overflow() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    for _i in 0..1000 {
        app.handle(Event::KeyDown(Key::A, KeyValue::Char('a')), &mut outputs);
    }

    app.handle(Event::KeyDown(Key::Enter, KeyValue::Enter), &mut outputs);
    assert_eq!(
        outputs.last_message,
        "sending message: aaaaaaaaaaaaaaaaaaaaaaaa"
    );
}

#[test]
fn button_a_sends_ping() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    app.handle(Event::ButtonDown(Button::A), &mut outputs);
    assert_eq!(outputs.net_tx.last_to_net, Some(ToNet::Ping));
}
