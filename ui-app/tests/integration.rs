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
    sent: Vec<ToNet>,
}

impl NetTx for MockNetTx {
    fn write(&mut self, to_net: &ToNet) {
        self.sent.push(to_net.clone());
    }
}

struct MockEeprom {
    data: [u8; 256],
}

impl Default for MockEeprom {
    fn default() -> Self {
        Self { data: [0; 256] }
    }
}

impl Eeprom for &mut MockEeprom {
    fn read(&mut self, data: &mut [u8; 256]) {
        data.copy_from_slice(&self.data);
    }

    fn write(&mut self, data: &[u8; 256]) {
        self.data.copy_from_slice(data);
    }
}

#[derive(Default)]
struct MockAudioControl {
    started: bool,
    enable_input: bool,
    enable_output: bool,
}

impl AudioControl for &mut MockAudioControl {
    fn start(&mut self) {
        self.started = true;
    }

    fn enable_input(&mut self, enabled: bool) {
        self.enable_input = enabled;
    }

    fn enable_output(&mut self, enabled: bool) {
        self.enable_output = enabled;
    }
}

#[derive(Default)]
struct MockAudioData {
    started: bool,
    read_count: u16,
    read: Vec<Frame>,
    written: Vec<Frame>,
}

impl AudioData for MockAudioData {
    async fn start(&mut self) {
        self.started = true;
    }

    async fn stop(&mut self) {
        self.started = true;
    }

    async fn read(&mut self) -> Frame {
        let frame = Frame([self.read_count; FRAME_SIZE]);
        self.read_count = self.read_count.wrapping_add(1);
        self.read.push(frame.clone());
        frame
    }

    async fn write(&mut self, frame: &Frame) {
        self.written.push(frame.clone());
    }
}

#[derive(Default)]
struct MockOutputs {
    button_a_down: bool,
    button_b_down: bool,
    status_led: MockLed,
    screen: MockScreen,
    net_tx: MockNetTx,
    eeprom: MockEeprom,
    last_message: String,
    audio_control: MockAudioControl,
    audio_data: MockAudioData,
}

impl Outputs for MockOutputs {
    fn button_a_down(&self) -> bool {
        self.button_a_down
    }

    fn button_b_down(&self) -> bool {
        self.button_b_down
    }

    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }

    fn screen(&mut self) -> &mut impl DrawTarget<Color = Rgb565> {
        &mut self.screen
    }

    fn net_tx(&mut self) -> &mut impl NetTx {
        &mut self.net_tx
    }

    fn eeprom(&mut self) -> impl Eeprom {
        &mut self.eeprom
    }

    fn audio_control(&mut self) -> impl AudioControl {
        &mut self.audio_control
    }

    fn audio_data(&mut self) -> &mut impl AudioData {
        &mut self.audio_data
    }

    fn log(&mut self, message: &str) {
        self.last_message = message.into();
    }
}

#[tokio::test]
async fn default_black() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[tokio::test]
async fn individual_buttons() {
    async fn individual_button(button: Button, color: Color) {
        let mut outputs = MockOutputs::default();
        let mut app = App::new();
        app.start(&mut outputs);
        assert_eq!(outputs.status_led.color, Some(Color::Black));

        // Up should have no effect
        app.handle(Event::ButtonUp(button), &mut outputs).await;
        assert_eq!(outputs.status_led.color, Some(Color::Black));

        // Pushing the button should illuminate the LED
        app.handle(Event::ButtonDown(button), &mut outputs).await;
        assert_eq!(outputs.status_led.color, Some(color));

        // Down should be idempotent
        app.handle(Event::ButtonDown(button), &mut outputs).await;
        assert_eq!(outputs.status_led.color, Some(color));

        // Up should extinguish the LED
        app.handle(Event::ButtonUp(button), &mut outputs).await;
        assert_eq!(outputs.status_led.color, Some(Color::Black));

        // Up should be idempotent
        app.handle(Event::ButtonUp(button), &mut outputs).await;
        assert_eq!(outputs.status_led.color, Some(Color::Black));
    }

    individual_button(Button::A, Color::Green).await;
    individual_button(Button::B, Color::Blue).await;
}

#[tokio::test]
async fn buttons_compose() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);
    assert_eq!(outputs.status_led.color, Some(Color::Black));

    app.handle(Event::ButtonDown(Button::B), &mut outputs).await;
    assert_eq!(outputs.status_led.color, Some(Color::Blue));

    app.handle(Event::ButtonDown(Button::A), &mut outputs).await;
    assert_eq!(outputs.status_led.color, Some(Color::Cyan));

    app.handle(Event::ButtonUp(Button::B), &mut outputs).await;
    assert_eq!(outputs.status_led.color, Some(Color::Green));

    app.handle(Event::ButtonUp(Button::A), &mut outputs).await;
    assert_eq!(outputs.status_led.color, Some(Color::Black));
}

#[tokio::test]
async fn key_logging() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    assert_eq!(outputs.last_message, "");

    app.handle(Event::KeyDown(Key::A, KeyValue::Char('a')), &mut outputs)
        .await;
    assert_eq!(outputs.last_message, "key down: A Char('a')");

    app.handle(Event::KeyUp(Key::A), &mut outputs).await;
    assert_eq!(outputs.last_message, "key up: A");
}

#[tokio::test]
async fn message_buffer_tolerates_overflow() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    for _i in 0..1000 {
        app.handle(Event::KeyDown(Key::A, KeyValue::Char('a')), &mut outputs)
            .await;
    }

    app.handle(Event::KeyDown(Key::Enter, KeyValue::Enter), &mut outputs)
        .await;
    assert_eq!(
        outputs.last_message,
        "sending message: aaaaaaaaaaaaaaaaaaaaaaaa"
    );
}

#[tokio::test]
async fn button_a_sends_ping() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    app.handle(Event::ButtonDown(Button::A), &mut outputs).await;
    assert_eq!(&outputs.net_tx.sent, &[ToNet::Ping]);
}

#[tokio::test]
async fn receive_ptt() {
    let mut outputs = MockOutputs::default();
    let mut app = App::new();
    app.start(&mut outputs);

    // Signal the start of audio
    app.handle(Event::FromNet(FromNet::AudioStart), &mut outputs)
        .await;
    assert_eq!(outputs.audio_control.started, true);
    assert_eq!(outputs.audio_control.enable_output, true);
    assert_eq!(outputs.audio_data.started, true);

    // Send a few frames
    let mut frames = vec![Frame::default(); 3];
    for (i, frame) in frames.iter_mut().enumerate() {
        frame.0.fill(i as u16);
        let from_net = Event::FromNet(FromNet::AudioFrame(frame.clone()));
        app.handle(from_net, &mut outputs).await;
    }

    assert_eq!(outputs.audio_data.written, frames);

    // Signal the end of audio
    app.handle(Event::FromNet(FromNet::AudioEnd), &mut outputs)
        .await;
    assert_eq!(outputs.audio_control.started, true);
    assert_eq!(outputs.audio_control.enable_output, false);
    // TODO re-enable
    // assert_eq!(outputs.audio_data.start, false);

    // Verify that out-of-context audio gets dropped
    let from_net = Event::FromNet(FromNet::AudioFrame(frames[0].clone()));
    app.handle(from_net, &mut outputs).await;

    assert_eq!(outputs.audio_data.written, frames);
    assert_eq!(
        outputs.last_message,
        "Dropped out-of-context message from NET chip"
    );
}
