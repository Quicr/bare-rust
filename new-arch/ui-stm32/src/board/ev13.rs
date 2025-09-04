use super::{Button, StatusLed};
use embassy_stm32::{
    bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    mode::Async,
    peripherals,
    usart::{self, Config, DataBits, Parity, StopBits, Uart, UartRx, UartTx},
};
use ui_app::{Led, Outputs, Write};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

pub struct SerialTx(UartTx<'static, Async>);
pub type SerialRx = UartRx<'static, Async>;

impl Write for SerialTx {
    fn write(&mut self, buf: &[u8]) -> usize {
        // TODO(RLB) Handle errors
        let _ = UartTx::write(&mut self.0, buf);
        buf.len()
    }
}

pub struct Board {
    status_led: StatusLed,
    mgmt_tx: SerialTx,
    pub ptt_button: Option<Button>,
    pub ai_button: Option<Button>,
    pub mgmt_rx: Option<SerialRx>,
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

        // MGMT UART
        let config = {
            let mut config = Config::default();
            config.baudrate = 115200;
            config.data_bits = DataBits::DataBits8;
            config.stop_bits = StopBits::STOP1;
            config.parity = Parity::ParityEven;
            config
        };
        let mgmt_uart = Uart::new(
            p.USART1, p.PA10, p.PA9, Irqs, p.DMA2_CH7, p.DMA2_CH5, config,
        )
        .unwrap();

        let (mgmt_tx, mgmt_rx) = mgmt_uart.split();

        Self {
            status_led,
            mgmt_tx: SerialTx(mgmt_tx),
            ptt_button: Some(ptt_button),
            ai_button: Some(ai_button),
            mgmt_rx: Some(mgmt_rx),
        }
    }
}

impl Outputs for Board {
    fn status_led(&mut self) -> &mut impl Led {
        &mut self.status_led
    }

    fn mgmt_tx(&mut self) -> &mut impl Write {
        &mut self.mgmt_tx
    }
}
