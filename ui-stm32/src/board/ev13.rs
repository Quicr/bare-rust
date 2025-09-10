use super::{Button, Keyboard, Screen, StatusLed};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    spi::Spi,
};
use ui_app::{Led, Outputs};

pub struct Board {
    status_led: StatusLed,
    pub screen: Screen, // XXX should not be pub
    pub ptt_button: Option<Button>,
    pub ai_button: Option<Button>,
    pub keyboard: Option<Keyboard>,
}

impl Board {
    pub fn new() -> Self {
        let config = {
            use embassy_stm32::{rcc::*, time::Hertz};

            let mut config = embassy_stm32::Config::default();

            config.rcc.hse = Some(Hse {
                freq: Hertz(6_000_000),
                mode: HseMode::Bypass,
            });
            config.rcc.sys = Sysclk::PLL1_P;
            config.rcc.pll_src = PllSource::HSE;
            config.rcc.pll = Some(Pll {
                prediv: PllPreDiv::DIV3,
                mul: PllMul::MUL168,
                divp: Some(PllPDiv::DIV2),
                divq: Some(PllQDiv::DIV7),
                divr: None,
            });

            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV4;
            config.rcc.apb2_pre = APBPrescaler::DIV2;
            config.rcc.ls = LsConfig {
                rtc: RtcClockSource::LSI,
                lsi: true,
                lse: None,
            };

            config
        };
        let p = embassy_stm32::init(config);

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

        // Screen
        let chip_select = Output::new(p.PB8, Level::Low, Speed::Low);
        let data_command = Output::new(p.PB9, Level::Low, Speed::Low);
        let reset = Output::new(p.PC13, Level::Low, Speed::Low);
        let backlight = Output::new(p.PC14, Level::Low, Speed::Low);

        let config = {
            use embassy_stm32::spi::*;
            let mut config = Config::default();
            config.mode.polarity = Polarity::IdleLow;
            config.mode.phase = Phase::CaptureOnFirstTransition;
            config.bit_order = BitOrder::MsbFirst;
            config
        };
        let spi1 = Spi::new_blocking_txonly(p.SPI1, p.PA5, p.PA7, config);
        let screen = Screen::new(chip_select, data_command, reset, backlight, spi1);

        // TODO(RLB): NET UART
        // TODO(RLB): MGMT UART

        Self {
            status_led,
            screen,
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

    fn screen(&mut self) -> &mut impl ui_app::Screen {
        &mut self.screen
    }

    fn log(&mut self, message: &str) {
        defmt::info!("{}", message);
    }
}
