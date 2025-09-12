use ui_app::*;

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

impl Screen for MockScreen {
    fn width(&self) -> usize {
        240
    }

    fn height(&self) -> usize {
        320
    }

    fn fill(&mut self, _color: u16) {
        // TODO: Store pixels
    }

    fn draw(&mut self, _left: usize, _right: usize, _top: usize, _bottom: usize, _data: &[u16]) {
        // TODO: Store pixels
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

    fn screen(&mut self) -> &mut impl Screen {
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
    assert_eq!(outputs.last_message, "key down");

    app.handle(Event::KeyUp(Key::A), &mut outputs);
    assert_eq!(outputs.last_message, "key up");
}

#[test]
fn button_a_sends_ping() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    app.handle(Event::ButtonDown(Button::A), &mut outputs);
    assert_eq!(outputs.net_tx.last_to_net, Some(ToNet::Ping));
}
