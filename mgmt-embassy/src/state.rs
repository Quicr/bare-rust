use defmt::info;
use embassy_stm32::{
    bind_interrupts,
    mode::Async,
    peripherals, usart,
    usart::{Config, DataBits, Parity, RingBufferedUartRx, StopBits, Uart, UartTx},
};
use embedded_io_async::Read;

use crate::commands::*;
use crate::drivers::{NetControl, RgbLed, UiControl};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    USART2 => usart::InterruptHandler<peripherals::USART2>;
    USART3_4 => usart::InterruptHandler<peripherals::USART3>;
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    Drop,
    Usb,
    Ui,
    Net,
    Command,
}

/// Routing configuration for each UART
pub struct UartRouting {
    pub usb: Interface,
    pub ui: Interface,
    pub net: Interface,
}

impl Default for UartRouting {
    // Default state:
    // * Process commands from USB rx
    // * Route NET and UI to USB tx
    fn default() -> Self {
        Self {
            usb: Interface::Command,
            ui: Interface::Usb,
            net: Interface::Usb,
        }
    }
}

pub struct State<'d> {
    // LEDs
    pub led_a: RgbLed,
    pub led_b: RgbLed,

    // Control registers for the UI chips
    pub ui_control: UiControl,
    pub net_control: NetControl,

    // UART connections
    pub usb_tx: UartTx<'static, Async>,
    pub usb_rx: RingBufferedUartRx<'d>,

    pub ui_tx: UartTx<'static, Async>,
    pub ui_rx: RingBufferedUartRx<'d>,

    pub net_tx: UartTx<'static, Async>,
    pub net_rx: RingBufferedUartRx<'d>,

    // Routing
    pub routing: UartRouting,
}

impl<'d> State<'d> {
    pub fn new(
        usb_rx_buf: &'d mut [u8],
        ui_rx_buf: &'d mut [u8],
        net_rx_buf: &'d mut [u8],
    ) -> Self {
        let p = embassy_stm32::init(Default::default());

        // LEDs
        let led_a = RgbLed::new(p.PA4, p.PA6, p.PA7);
        let led_b = RgbLed::new(p.PB0, p.PB6, p.PB15);

        // Control registers for the UI chips
        let ui_control = UiControl::new(p.PB3, p.PA15, p.PB8);
        let net_control = NetControl::new(p.PB4, p.PB5);

        // UART to USB
        let mut config = Config::default();
        config.baudrate = 115200;
        config.data_bits = DataBits::DataBits8;
        config.stop_bits = StopBits::STOP1;
        config.parity = Parity::ParityNone;

        let (usb_tx, usb_rx) = {
            let (tx, rx) = Uart::new(
                p.USART1, p.PA10, p.PA9, Irqs, p.DMA1_CH2, p.DMA1_CH3, config,
            )
            .unwrap()
            .split();

            (tx, rx.into_ring_buffered(usb_rx_buf))
        };

        // UART to UI
        let (ui_tx, ui_rx) = {
            let (tx, rx) = Uart::new(p.USART2, p.PA3, p.PA2, Irqs, p.DMA1_CH4, p.DMA1_CH5, config)
                .unwrap()
                .split();

            (tx, rx.into_ring_buffered(ui_rx_buf))
        };

        // UART to NET
        let (net_tx, net_rx) = {
            let (tx, rx) = Uart::new(
                p.USART3, p.PB11, p.PB10, Irqs, p.DMA1_CH7, p.DMA1_CH6, config,
            )
            .unwrap()
            .split();

            (tx, rx.into_ring_buffered(net_rx_buf))
        };

        Self {
            led_a,
            led_b,
            ui_control,
            net_control,
            usb_tx,
            usb_rx,
            ui_tx,
            ui_rx,
            net_tx,
            net_rx,
            routing: Default::default(),
        }
    }

    pub async fn route_data(&mut self, src: Interface, buf: &mut [u8]) {
        let dst = match src {
            Interface::Usb => self.routing.usb,
            Interface::Ui => self.routing.ui,
            Interface::Net => self.routing.net,
            _ => unreachable!("Invalid source interface"),
        };

        // Command data is handled separately
        if dst == Interface::Command {
            return;
        }

        let n = match src {
            Interface::Usb => self.usb_rx.read(buf).await.unwrap(),
            Interface::Ui => self.ui_rx.read(buf).await.unwrap(),
            Interface::Net => self.net_rx.read(buf).await.unwrap(),
            _ => unreachable!("Invalid source interface"),
        };

        // No data to process
        if n == 0 {
            return;
        }

        let data = &buf[..n];

        match dst {
            Interface::Drop => {}
            Interface::Usb => self.usb_tx.write(&data).await.unwrap(),
            Interface::Ui => self.ui_tx.write(&data).await.unwrap(),
            Interface::Net => self.net_tx.write(&data).await.unwrap(),
            Interface::Command => unreachable!("Invalid destination interface"),
        }
    }

    pub async fn handle_command(&mut self, buf: &mut [u8]) {
        let mut type_len = [0u8; 5];
        self.usb_rx.read_exact(&mut type_len).await.unwrap();

        let Ok(command) = Command::try_from(type_len[0]) else {
            defmt::warn!("Invalid command: {}", type_len[0]);
            return;
        };

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&type_len[1..]);
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len == 0 {
            self.direct_command(command).await;
        } else {
            self.forwarding_command(command, len, buf).await;
        }
    }

    async fn direct_command(&mut self, command: Command) {
        match command {
            Command::Version => self.usb_tx.write(VERSION).await.unwrap(),
            Command::WhoAreYou => self.usb_tx.write(HELLO_I_AM_A_HACTAR_DEVICE).await.unwrap(),
            Command::HardReset => {
                info!("Hard reset requested");

                // Reset both chips
                self.reset_ui().await;
                self.reset_net().await;

                // Reset routing to defaults (Debug mode: logs enabled)
                self.routing = UartRouting::default();

                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::Reset => {
                self.reset_ui().await;
                self.reset_net().await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::ResetUi => {
                self.reset_ui().await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::ResetNet => {
                self.reset_net().await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::FlashUi => {
                info!("Entering UI flash mode");

                // Hold NET in reset
                self.net_control.hold_in_reset();

                // Configure routing: USB->UI, UI->USB, NET->None
                self.routing.usb = Interface::Ui;
                self.routing.ui = Interface::Usb;
                self.routing.net = Interface::Drop;

                // Send OK byte
                let _ = self.usb_tx.write(&[OK_BYTE]).await;

                // Reconfigure the USB interface
                let config = {
                    let mut config = Config::default();
                    config.baudrate = 115200;
                    config.data_bits = DataBits::DataBits9;
                    config.stop_bits = StopBits::STOP1;
                    config.parity = Parity::ParityEven;
                    config
                };

                self.usb_rx.set_config(&config).unwrap();
                self.usb_tx.set_config(&config).unwrap();

                // Delay to allow UART reconfiguration to settle
                embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;

                // Put UI chip into bootloader mode
                self.ui_control.bootloader_mode();

                // Send Ready byte
                let _ = self.usb_tx.write(&[READY_BYTE]).await;

                info!("UI flash mode active - bootloader ready");
            }
            Command::FlashNet => {
                info!("Entering NET flash mode");

                // Hold UI in reset
                self.ui_control.hold_in_reset();

                // Configure routing: USB->NET, NET->USB, UI->None
                self.routing.usb = Interface::Net;
                self.routing.net = Interface::Usb;
                self.routing.ui = Interface::Drop;

                // Send OK byte
                let _ = self.usb_tx.write(&[OK_BYTE]).await;

                // Delay before entering bootloader mode
                embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;

                // Put NET chip into bootloader mode
                self.net_control.bootloader_mode();

                // Send Ready byte
                let _ = self.usb_tx.write(&[READY_BYTE]).await;
                info!("NET flash mode active - bootloader ready");
            }
            Command::EnableLogs => {
                self.enable_logs_ui(true).await;
                self.enable_logs_net(true).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::EnableLogsUi => {
                self.enable_logs_ui(true).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::EnableLogsNet => {
                self.enable_logs_net(true).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::DisableLogs => {
                self.enable_logs_ui(false).await;
                self.enable_logs_net(false).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::DisableLogsUi => {
                self.enable_logs_ui(false).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::DisableLogsNet => {
                self.enable_logs_net(false).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }
            Command::DefaultLogging => {
                self.enable_logs_ui(true).await;
                self.enable_logs_net(true).await;
                self.usb_tx.write(OK_ASCII).await.unwrap()
            }

            // When these commands are sent with zero data, they are just a noop
            Command::ToUsb => {}
            Command::ToUi => {}
            Command::ToNet => {}
        }
    }

    async fn forwarding_command(&mut self, command: Command, len: usize, buf: &mut [u8]) {
        let mut remaining = len;
        while remaining != 0 {
            let curr_len = buf.len().min(remaining);
            let curr = &mut buf[..curr_len];

            self.usb_rx.read_exact(curr).await.unwrap();

            match command {
                Command::ToUsb => {
                    self.led_b.toggle_red();
                    self.usb_tx.write(buf).await.unwrap();
                }
                Command::ToUi => {
                    self.led_b.toggle_blue();
                    self.ui_tx.write(buf).await.unwrap();
                }
                Command::ToNet => {
                    self.led_b.toggle_green();
                    self.net_tx.write(buf).await.unwrap();
                }
                _ => unreachable!("Invalid forwarding command"),
            }

            remaining -= curr_len
        }
    }

    pub async fn reset_ui(&mut self) {
        info!("Resetting UI chip");
        self.ui_control.normal_mode();
    }

    pub async fn reset_net(&mut self) {
        info!("Resetting NET chip");
        self.net_control.normal_mode();
    }

    pub async fn enable_logs_ui(&mut self, enabled: bool) {
        info!("Enabling UI logs");
        self.routing.ui = if enabled {
            Interface::Usb
        } else {
            Interface::Drop
        };
    }

    pub async fn enable_logs_net(&mut self, enabled: bool) {
        info!("Enabling NET logs");
        self.routing.net = if enabled {
            Interface::Usb
        } else {
            Interface::Drop
        };
    }
}
