use super::{Button, Keyboard, NetTx, Screen, StatusLed};
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    i2c::{mode::Master, I2c},
    mode::{Async, Blocking},
    peripherals,
    spi::Spi,
    usart::{self, UartRx, UartTx},
};
use ui_app::{Led, Outputs};

pub struct Board {
    status_led: StatusLed,
    screen: Screen,
    net_tx: NetTx<UartTx<'static, Async>>,
    pub i2c: I2c<'static, Blocking, Master>,
    pub button_a: Option<Button>,
    pub button_b: Option<Button>,
    pub keyboard: Option<Keyboard>,
    pub net_rx: Option<UartRx<'static, Async>>,
}

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

impl Board {
    pub async fn new() -> Self {
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

            // XXX(RLB) The prediv = M value here must be the same as the PLL config above.  The
            // CubeMX clock tree shows one M value for both PLLs.
            config.rcc.plli2s = Some(Pll {
                prediv: PllPreDiv::DIV3,
                mul: PllMul::MUL50,
                divp: None,
                divq: None,
                divr: Some(PllRDiv::DIV2),
            });

            config
        };
        let p = embassy_stm32::init(config);

        // Status LED
        let r = Output::new(p.PA4, Level::Low, Speed::Low);
        let g = Output::new(p.PC5, Level::Low, Speed::Low);
        let b = Output::new(p.PB3, Level::Low, Speed::Low);
        let status_led = StatusLed { r, g, b };

        // Buttons
        let button_a = ExtiInput::new(p.PC1, p.EXTI1, Pull::Up);
        let button_b = ExtiInput::new(p.PC0, p.EXTI0, Pull::Up);

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
        let screen = Screen::new(chip_select, data_command, reset, backlight, spi1).await;

        // NET UART
        let net_uart = {
            use embassy_stm32::usart::*;
            let mut config = Config::default();
            config.baudrate = 460800;
            config.data_bits = DataBits::DataBits8;
            config.stop_bits = StopBits::STOP2;
            config.parity = Parity::ParityNone;

            Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH6, p.DMA1_CH5, config).unwrap()
        };

        let (net_tx, net_rx) = net_uart.split();
        let net_tx = NetTx::new(net_tx);

        // EEPROM I2C interface
        let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, Default::default());

        Self {
            status_led,
            screen,
            net_tx,
            i2c,
            button_a: Some(button_a),
            button_b: Some(button_b),
            keyboard: Some(keyboard),
            net_rx: Some(net_rx),
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

    fn net_tx(&mut self) -> &mut impl ui_app::NetTx {
        &mut self.net_tx
    }

    fn log(&mut self, message: &str) {
        defmt::info!("{}", message);
    }
}
