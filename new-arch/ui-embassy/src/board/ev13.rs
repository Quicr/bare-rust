use super::{Button, StatusLed};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
};
use ui_app::{Led, Outputs};

pub struct Board {
    status_led: StatusLed,
    pub ptt_button: Option<Button>,
    pub ai_button: Option<Button>,
}

impl Board {
    pub fn new() -> Self {
        let p = embassy_stm32::init(Default::default());

        let r = Output::new(p.PA4, Level::Low, Speed::Low);
        let g = Output::new(p.PC5, Level::Low, Speed::Low);
        let b = Output::new(p.PB3, Level::Low, Speed::Low);
        let status_led = StatusLed { r, g, b };

        let ai_button = ExtiInput::new(p.PC0, p.EXTI0, Pull::Up);
        let ptt_button = ExtiInput::new(p.PC1, p.EXTI1, Pull::Up);

        Self {
            status_led,
            ptt_button: Some(ptt_button),
            ai_button: Some(ai_button),
        }
    }
}

impl Outputs for Board {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }
}
