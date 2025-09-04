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

#[derive(Default)]
struct MockOutputs {
    status_led: MockLed,
}

impl Outputs for MockOutputs {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }
}

#[test]
fn default_black() {
    let mut outputs = MockOutputs::default();
    let _ = App::start(&mut outputs);

    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[test]
fn ptt_button_green() {
    let mut outputs = MockOutputs::default();
    let mut app = App::start(&mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    // Up should have no effect
    app.handle(Event::PttUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    // Pushing the button should illuminate the LED
    app.handle(Event::PttDown, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Green));

    // Down should be idempotent
    app.handle(Event::PttDown, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Green));

    // Up should extinguish the LED
    app.handle(Event::PttUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    // Up should be idempotent
    app.handle(Event::PttUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[test]
fn ai_button_blue() {
    let mut outputs = MockOutputs::default();
    let mut app = App::start(&mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    // Up should have no effect
    app.handle(Event::AiUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    // Pushing the button should illuminate the LED
    app.handle(Event::AiDown, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Blue));

    // Down should be idempotent
    app.handle(Event::AiDown, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Blue));

    // Up should extinguish the LED
    app.handle(Event::AiUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    // Up should be idempotent
    app.handle(Event::AiUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[test]
fn buttons_compose() {
    let mut outputs = MockOutputs::default();
    let mut app = App::start(&mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    app.handle(Event::AiDown, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Blue));

    app.handle(Event::PttDown, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Cyan));

    app.handle(Event::AiUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Green));

    app.handle(Event::PttUp, &mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));
}
