use heapless::mpmc::Q32;
use stm32f4xx_hal::{
    gpio::{self, Input, Output, Pin, PushPull},
    pac::{Peripherals, TIM2, TIM3},
    prelude::*,
    timer::{self, CounterMs, Event},
};

use super::traits::{self, *};

impl<const P: char, const N: u8> GpioOutput for Pin<P, N, Output<PushPull>> {
    fn set_low(&mut self) {
        Pin::set_low(self);
    }

    fn set_high(&mut self) {
        Pin::set_high(self);
    }

    fn is_set_low(&self) -> bool {
        Pin::is_set_low(self)
    }

    fn is_set_high(&self) -> bool {
        Pin::is_set_high(self)
    }
}

impl<const P: char, const N: u8> GpioInput for Pin<P, N, Input> {
    fn is_low(&self) -> bool {
        Pin::is_low(self)
    }
}

impl<T> TimerReset for CounterMs<T>
where
    T: timer::Instance,
{
    fn reset(&mut self) {
        self.clear_interrupt(Event::Update);
    }
}

pub struct Signal {
    signal_id: SignalId,
    events: Option<BoardEventSender>,
}

impl Signal {
    pub fn new(signal_id: SignalId) -> Self {
        Self {
            signal_id,
            events: None,
        }
    }
}

impl Source<BoardEventSender> for Signal {
    fn connect(&mut self, events: BoardEventSender) {
        self.events = Some(events);
    }
}

impl traits::Signal<BoardEventSender> for Signal {
    fn fire(&mut self) {
        let Some(events) = &self.events else {
            return;
        };

        let event = BoardEvent::Signal(self.signal_id);

        // TODO(RLB): Report failiure
        let _ = events.try_send(event);
    }
}

pub struct Button<P>
where
    P: GpioInput,
{
    button_id: ButtonId,
    events: Option<BoardEventSender>,
    pin: P,
    is_low: bool,
}

impl<P> Button<P>
where
    P: GpioInput,
{
    pub fn new(pin: P, button_id: ButtonId) -> Self {
        let is_low = pin.is_low();
        Self {
            button_id,
            events: None,
            pin,
            is_low,
        }
    }
}

impl<P> traits::Button<BoardEventSender> for Button<P>
where
    P: GpioInput,
{
    fn scan(&mut self) {
        let was_low = self.is_low;
        self.is_low = self.pin.is_low();

        if self.is_low == was_low {
            return;
        }

        let Some(events) = &self.events else {
            return;
        };

        let event = match self.is_low {
            true => BoardEvent::Button(self.button_id, ButtonState::Down),
            false => BoardEvent::Button(self.button_id, ButtonState::Up),
        };

        // TODO(RLB): Report failiure
        let _ = events.try_send(event);
    }
}

impl<P> Source<BoardEventSender> for Button<P>
where
    P: GpioInput,
{
    fn connect(&mut self, events: BoardEventSender) {
        self.events = Some(events);
    }
}

static BOARD_EVENT_QUEUE: Q32<BoardEvent> = Q32::new();

pub struct BoardEventSender;

impl Sender<BoardEvent> for BoardEventSender {
    fn try_send(&self, event: BoardEvent) -> Result<(), BoardEvent> {
        BOARD_EVENT_QUEUE.enqueue(event)
    }
}

// XXX This is cheating a little; we should do moves always
#[derive(Copy, Clone)]
pub struct BoardEventReceiver;

impl Receiver<BoardEvent> for BoardEventReceiver {
    fn try_recv(&self) -> Option<BoardEvent> {
        BOARD_EVENT_QUEUE.dequeue()
    }
}

type ButtonA = Button<gpio::PC1<Input>>;
type ButtonB = Button<gpio::PC0<Input>>;
type GreenLedSignal = Signal;
type ScanInputsTimer = CounterMs<TIM2>;
type GreenLedTimer = CounterMs<TIM3>;
type RedLed = gpio::PA6<Output<PushPull>>;
type GreenLed = gpio::PC5<Output<PushPull>>;
type BlueLed = gpio::PA1<Output<PushPull>>;

pub struct Board {
    button_a: Option<ButtonA>,
    button_b: Option<ButtonB>,
    green_led_signal: Option<GreenLedSignal>,
    scan_inputs_timer: Option<ScanInputsTimer>,
    green_led_timer: Option<GreenLedTimer>,
    red_led: Option<RedLed>,
    green_led: Option<GreenLed>,
    blue_led: Option<BlueLed>,
}

impl Board {
    pub fn new(dp: Peripherals) -> Self {
        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // Create inputs
        let button_a_pin = gpioc.pc1.into_pull_up_input();
        let mut button_a = Button::new(button_a_pin, ButtonId::A);
        button_a.connect(BoardEventSender);

        let button_b_pin = gpioc.pc0.into_pull_up_input();
        let mut button_b = Button::new(button_b_pin, ButtonId::B);
        button_b.connect(BoardEventSender);

        let mut green_led_signal = Signal::new(SignalId::GreenLedTimer);
        green_led_signal.connect(BoardEventSender);

        // Create outputs
        let mut red_led = gpioa.pa6.into_push_pull_output();
        red_led.set_high();

        let mut green_led = gpioc.pc5.into_push_pull_output();
        green_led.set_high();

        let mut blue_led = gpioa.pa1.into_push_pull_output();
        blue_led.set_high();

        // Create timers
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(24.MHz()).freeze();

        let mut scan_inputs_timer = dp.TIM2.counter_ms(&clocks);
        scan_inputs_timer.start(50.millis()).unwrap();
        scan_inputs_timer.listen(Event::Update);

        let mut green_led_timer = dp.TIM3.counter_ms(&clocks);
        green_led_timer.start(1000.millis()).unwrap();
        green_led_timer.listen(Event::Update);

        Self {
            button_a: Some(button_a),
            button_b: Some(button_b),
            green_led_signal: Some(green_led_signal),
            scan_inputs_timer: Some(scan_inputs_timer),
            green_led_timer: Some(green_led_timer),
            red_led: Some(red_led),
            green_led: Some(green_led),
            blue_led: Some(blue_led),
        }
    }
}

impl traits::Board for Board {
    type EventSender = BoardEventSender;
    type EventReceiver = BoardEventReceiver;

    type ButtonA = ButtonA;
    type ButtonB = ButtonB;
    type ScanInputsTimer = ScanInputsTimer;
    type GreenLedTimer = GreenLedTimer;
    type GreenLedSignal = GreenLedSignal;

    type RedLed = RedLed;
    type GreenLed = GreenLed;
    type BlueLed = BlueLed;

    fn events(&mut self) -> Self::EventReceiver {
        BoardEventReceiver
    }

    fn button_a(&mut self) -> Self::ButtonA {
        self.button_a.take().unwrap()
    }

    fn button_b(&mut self) -> Self::ButtonB {
        self.button_b.take().unwrap()
    }

    fn scan_inputs_timer(&mut self) -> Self::ScanInputsTimer {
        self.scan_inputs_timer.take().unwrap()
    }

    fn green_led_timer(&mut self) -> Self::GreenLedTimer {
        self.green_led_timer.take().unwrap()
    }

    fn green_led_signal(&mut self) -> Self::GreenLedSignal {
        self.green_led_signal.take().unwrap()
    }

    fn red_led(&mut self) -> Self::RedLed {
        self.red_led.take().unwrap()
    }

    fn green_led(&mut self) -> Self::GreenLed {
        self.green_led.take().unwrap()
    }

    fn blue_led(&mut self) -> Self::BlueLed {
        self.blue_led.take().unwrap()
    }
}
