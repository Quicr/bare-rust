use embassy_stm32::{
    gpio::Output,
    mode::Blocking,
    spi::{Spi, Word},
};
use embassy_time::Timer;
use num_enum::IntoPrimitive;

pub struct Screen {
    chip_select: Output<'static>,
    data_command: Output<'static>,
    reset: Output<'static>,
    backlight: Output<'static>,
    spi: Spi<'static, Blocking>,
}

impl ui_app::Screen for Screen {
    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }

    fn fill(&mut self, color: u16) {
        let row = [color; Self::WIDTH];

        self.set_window(0, 0, Self::WIDTH, Self::HEIGHT);
        self.send_command(Command::WriteMemory, &[]);
        for _ in 0..Self::HEIGHT {
            self.spi.blocking_write(&row).unwrap();
        }
    }

    fn draw(&mut self, left: usize, right: usize, top: usize, bottom: usize, data: &[u16]) {
        self.set_window(left, right, top, bottom);
        self.send_command(Command::WriteMemory, &[]);
        self.send_data(data);
    }
}

impl Screen {
    const WIDTH: usize = 320;
    const HEIGHT: usize = 240;

    pub fn new(
        chip_select: Output<'static>,
        data_command: Output<'static>,
        reset: Output<'static>,
        backlight: Output<'static>,
        spi: Spi<'static, Blocking>,
    ) -> Self {
        Self {
            chip_select,
            data_command,
            reset,
            backlight,
            spi,
        }
    }

    // Initial command sequence borrowed from
    //
    // https://github.com/yuri91/ili9341-rs/blob/master/src/lib.rs#L139
    //
    // It might be better to put this in new(), as is done there.
    pub async fn init(&mut self) {
        // Chip select pin is always held low
        defmt::debug!("chip_select");
        self.chip_select.set_low();

        // Do hardware reset by holding reset low for at least 10us, then wait 5ms for the reset to
        // occur before sending more commands.
        defmt::debug!("reset low");
        self.reset.set_low();
        Timer::after_millis(200).await;
        defmt::debug!("reset high");
        self.reset.set_high();
        Timer::after_millis(200).await;

        // Do hardware reset, then wait 120ms for the reset to occur before sending more commands.
        defmt::debug!("software reset");
        self.send_command(Command::SoftwareReset, &[]);
        Timer::after_millis(200).await;

        // Set portrait orientation
        defmt::debug!("set orientation");
        self.send_command(
            Command::MemoryAccessControl,
            &[u8::from(Orientation::Portrait)],
        );

        // Set the pixel format to rgb565
        defmt::debug!("set pixel format");
        self.send_command(Command::SetPixelFormat, &[0x55]);

        // Have the display emerge from sleep, then wait 5ms for it to wake up
        defmt::debug!("set sleep mode off");
        self.send_command(Command::SleepModeOff, &[]);
        Timer::after_millis(200).await;

        // Turn the display on
        defmt::debug!("display on");
        self.send_command(Command::DisplayOn, &[]);

        // Turn on the backlight
        defmt::debug!("backlight on");
        self.set_backlight(true);
    }

    fn set_window(&mut self, x0: usize, x1: usize, y0: usize, y1: usize) {
        let mut x_data = [0u8; 4];
        x_data[..2].copy_from_slice(&(x0 as u16).to_be_bytes());
        x_data[2..].copy_from_slice(&(x1 as u16).to_be_bytes());

        let mut y_data = [0u8; 4];
        y_data[..2].copy_from_slice(&(y0 as u16).to_be_bytes());
        y_data[2..].copy_from_slice(&(y1 as u16).to_be_bytes());

        self.send_command(Command::SetColumnAddress, &x_data);
        self.send_command(Command::SetPageAddress, &y_data);
    }

    fn send_command(&mut self, cmd: Command, data: &[u8]) {
        self.data_command.set_low();
        self.spi.blocking_write(&[u8::from(cmd)]).unwrap(); // TODO propagate error

        self.send_data(data);
    }

    fn send_data<W: Word>(&mut self, data: &[W]) {
        self.data_command.set_high();
        self.spi.blocking_write(data).unwrap(); // TODO propagate error
    }

    pub fn set_backlight(&mut self, on: bool) {
        self.backlight.set_level(on.into());
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, IntoPrimitive)]
#[repr(u8)]
enum Command {
    SetColumnAddress = 0x2a,
    ContentAdaptiveBrightness = 0x55,
    DisplayOff = 0x28,
    DisplayOn = 0x29,
    IdleModeFrameRate = 0xb2,
    IdleModeOff = 0x38,
    IdleModeOn = 0x39,
    InvertOff = 0x20,
    InvertOn = 0x21,
    MemoryAccessControl = 0x36,
    WriteMemory = 0x2c,
    NormalModeFrameRate = 0xb1,
    SetPageAddress = 0x2b,
    SetPixelFormat = 0x3a,
    SetBrightness = 0x51,
    SleepModeOff = 0x11,
    SleepModeOn = 0x10,
    SoftwareReset = 0x01,
    VerticalScrollAddr = 0x37,
    DefineVerticalScroll = 0x33,
}

#[allow(dead_code)]
#[derive(Copy, Clone, IntoPrimitive)]
#[repr(u8)]
enum Orientation {
    Portrait = 0x40 | 0x08,
    Landscape = 0x20 | 0x08,
    PortraitFlipped = 0x80 | 0x08,
    LandscapeFlipped = 0x40 | 0x80 | 0x20 | 0x08,
}
