use super::{Button, Eeprom, Keyboard, NetTx, StatusLed};
use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
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
use embassy_sync::blocking_mutex::{raw::NoopRawMutex, NoopMutex};
use embassy_time::Delay;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{ColorOrder, Orientation},
    Builder, Display,
};
use static_cell::StaticCell;
use ui_app::{Led, Outputs};

type DisplaySpiDevice = SpiDevice<'static, NoopRawMutex, Spi<'static, Blocking>, Output<'static>>;
type DisplaySpiInterface = SpiInterface<'static, DisplaySpiDevice, Output<'static>>;

pub struct Board {
    status_led: StatusLed,
    screen: Display<DisplaySpiInterface, ST7789, Output<'static>>,
    net_tx: NetTx<UartTx<'static, Async>>,
    i2c: I2c<'static, Blocking, Master>,
    pub button_a: Option<Button>,
    pub button_b: Option<Button>,
    pub keyboard: Option<Keyboard>,
    pub net_rx: Option<UartRx<'static, Async>>,

    // This field isn't used, but is held to keep the pin low instead of floating
    #[allow(dead_code)]
    backlight: Output<'static>,
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
        static SPI_BUS: StaticCell<NoopMutex<RefCell<Spi<'static, Blocking>>>> = StaticCell::new();
        let display_buffer = cortex_m::singleton!(: [u8; 512] = [0; 512]).unwrap();

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
        let spi = Spi::new_blocking_txonly(p.SPI1, p.PA5, p.PA7, config);
        let spi_bus = NoopMutex::new(RefCell::new(spi));
        let spi_bus = SPI_BUS.init(spi_bus);
        let spi_dev = SpiDevice::new(spi_bus, chip_select);
        let spi_if = SpiInterface::new(spi_dev, data_command, display_buffer);

        let screen = Builder::new(ST7789, spi_if)
            .reset_pin(reset)
            .orientation(Orientation::new().flip_horizontal())
            .color_order(ColorOrder::Bgr)
            .init(&mut Delay)
            .unwrap();

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

        // I2C interface for EEPROM and audio chip control
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
            backlight,
        }
    }
}

impl Outputs for Board {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }

    fn screen(&mut self) -> &mut impl DrawTarget<Color = Rgb565> {
        &mut self.screen
    }

    fn net_tx(&mut self) -> &mut impl ui_app::NetTx {
        &mut self.net_tx
    }

    fn eeprom(&mut self) -> impl ui_app::Eeprom {
        Eeprom { i2c: &mut self.i2c }
    }

    fn log(&mut self, message: &str) {
        defmt::info!("{}", message);
    }
}
