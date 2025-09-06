use super::{Button, Keyboard, StatusLed};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
};
use ui_app::{Led, Outputs};

pub struct Board {
    status_led: StatusLed,
    pub ptt_button: Option<Button>,
    pub ai_button: Option<Button>,
    pub keyboard: Option<Keyboard>,
}

impl Board {
    pub fn new() -> Self {
        let p = embassy_stm32::init(Default::default());

        // Status LED
        let r = Output::new(p.PA4, Level::Low, Speed::Low);
        let g = Output::new(p.PC5, Level::Low, Speed::Low);
        let b = Output::new(p.PB3, Level::Low, Speed::Low);
        let status_led = StatusLed { r, g, b };

        // Buttons
        let ai_button = ExtiInput::new(p.PC0, p.EXTI0, Pull::Up);
        let ptt_button = ExtiInput::new(p.PC1, p.EXTI1, Pull::Up);

        // Keyboard
        let cols = [
            Output::new(p.PB13, Level::Low, Speed::Low),
            Output::new(p.PB15, Level::Low, Speed::Low),
            Output::new(p.PC6, Level::Low, Speed::Low),
            Output::new(p.PC7, Level::Low, Speed::Low),
            Output::new(p.PC9, Level::Low, Speed::Low),
        ];
        let rows = [
            Input::new(p.PB12, Pull::Down),
            Input::new(p.PB14, Pull::Down),
            Input::new(p.PC8, Pull::Down),
            Input::new(p.PA8, Pull::Down),
            Input::new(p.PB0, Pull::Down),
            Input::new(p.PB1, Pull::Down),
            Input::new(p.PB11, Pull::Down),
        ];
        let keyboard = Keyboard::new(cols, rows);

        Self {
            status_led,
            ptt_button: Some(ptt_button),
            ai_button: Some(ai_button),
            keyboard: Some(keyboard),
        }
    }
}

impl Outputs for Board {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }

    fn log(&mut self, message: &str) {
        defmt::info!("{}", message);
    }
}
