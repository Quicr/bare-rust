use super::{Button, Keyboard, NetTx, StatusLed};
use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Input, Level, Output, Pull, Speed},
    mode::{Async, Blocking},
    peripherals,
    spi::{Spi, Word},
    usart::{self, UartRx, UartTx},
};
use embassy_time::Delay;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use ili9341::{Ili9341, Orientation};
use ui_app::{Led, Outputs};

struct DisplayData {
    // These fields are not used, but we need to keep them alive so that the chip select output is
    // held low and the backlight output is held high.
    _chip_select: Output<'static>,
    _backlight: Output<'static>,

    data_command: Output<'static>,
    spi: Spi<'static, Blocking>,
}

impl DisplayData {
    fn new(
        mut backlight: Output<'static>,
        mut chip_select: Output<'static>,
        data_command: Output<'static>,
        spi: Spi<'static, Blocking>,
    ) -> Self {
        backlight.set_high();
        chip_select.set_low();

        Self {
            _backlight: backlight,
            _chip_select: chip_select,
            data_command,
            spi,
        }
    }

    fn write(&mut self, data: DataFormat<'_>) -> Result<(), DisplayError> {
        use DataFormat::*;
        match data {
            U8(slice) => self.write_slice(slice),
            U16(slice) => self.write_slice(slice),
            U16BE(slice) => self.write_slice(slice),
            U16LE(slice) => self.write_slice(slice),
            U8Iter(iter) => self.write_iter(iter),
            U16BEIter(iter) => self.write_iter(iter),
            U16LEIter(iter) => self.write_iter(iter),
            _ => unreachable!(),
        }
    }

    fn write_slice<W: Word>(&mut self, data: &[W]) -> Result<(), DisplayError> {
        self.spi.blocking_write(data).unwrap();
        Ok(())
    }

    fn write_iter<W: Word>(
        &mut self,
        iter: &mut dyn Iterator<Item = W>,
    ) -> Result<(), DisplayError> {
        const CHUNK_SIZE: usize = 128;

        // XXX(RLB) Very C-style iteration, could probably write this in a way that would optimize
        // better.
        let mut data = [W::default(); CHUNK_SIZE];
        let mut n = 0;
        for (i, x) in iter.enumerate() {
            data[i % CHUNK_SIZE] = x;
            n = i + 1;

            if n > 0 && n % CHUNK_SIZE == 0 {
                self.spi.blocking_write(&data).unwrap();
                n = 0;
            }
        }

        self.spi.blocking_write(&data[..n]).unwrap();
        Ok(())
    }
}

impl WriteOnlyDataCommand for DisplayData {
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        self.data_command.set_low();
        self.write(cmd)
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        self.data_command.set_high();
        self.write(buf)
    }
}

pub struct Board {
    status_led: StatusLed,
    screen: Ili9341<DisplayData, Output<'static>>,
    net_tx: NetTx<UartTx<'static, Async>>,
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

            config
        };
        let p = embassy_stm32::init(config);

        // Status LED
        let r = Output::new(p.PA6, Level::Low, Speed::Low);
        let g = Output::new(p.PC5, Level::Low, Speed::Low);
        let b = Output::new(p.PA1, Level::Low, Speed::Low);
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
        let spi = Spi::new_blocking_txonly(p.SPI1, p.PA5, p.PA7, config);

        let screen = Ili9341::new(
            DisplayData::new(backlight, chip_select, data_command, spi),
            reset,
            &mut Delay,
            Orientation::Portrait,
            ili9341::DisplaySize240x320,
        )
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

        Self {
            status_led,
            screen,
            net_tx,
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

    fn screen(&mut self) -> &mut impl DrawTarget<Color = Rgb565> {
        &mut self.screen
    }

    fn net_tx(&mut self) -> &mut impl ui_app::NetTx {
        &mut self.net_tx
    }

    fn log(&mut self, message: &str) {
        defmt::info!("{}", message);
    }
}
