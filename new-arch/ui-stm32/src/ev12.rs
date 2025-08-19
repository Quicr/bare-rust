use ui_app::*;

use stm32f4xx_hal::{gpio::*, prelude::*};

struct StatusLed {
    r: PA6<Output>,
    g: PC5<Output>,
    b: PA1<Output>,
}

impl Led for StatusLed {
    fn set(&mut self, r: bool, g: bool, b: bool) {
        self.r.set_state((!r).into());
        self.g.set_state((!g).into());
        self.b.set_state((!b).into());
    }
}

pub type PttButton = PC1; // Top button
pub type AiButton = PC0; // Bottom button

pub struct Board {
    status_led: StatusLed,
    pub ptt_button: Option<PttButton>,
    pub ai_button: Option<AiButton>,
}

impl Board {
    pub fn new() -> Self {
        let mut dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();

        let rcc = dp.RCC.constrain();
        rcc.cfgr.use_hse(16.MHz()).freeze();

        let mut syscfg = dp.SYSCFG.constrain();

        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // Configure the status LEDs
        let r = gpioa.pa6.into_push_pull_output_in_state(PinState::High);
        let g = gpioc.pc5.into_push_pull_output_in_state(PinState::High);
        let b = gpioa.pa1.into_push_pull_output_in_state(PinState::High);
        let status_led = StatusLed { r, g, b };

        // Configure the buttons to drive interrupts
        let mut ptt_button = gpioc.pc1.into_pull_up_input();
        ptt_button.make_interrupt_source(&mut syscfg);
        ptt_button.enable_interrupt(&mut dp.EXTI);
        ptt_button.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);

        let mut ai_button = gpioc.pc0.into_pull_up_input();
        ai_button.make_interrupt_source(&mut syscfg);
        ai_button.enable_interrupt(&mut dp.EXTI);
        ai_button.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);

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
