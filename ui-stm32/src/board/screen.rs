use core::convert::TryFrom;
use embassy_stm32::{gpio::Output, mode::Blocking, spi::Spi};
use embassy_time::Timer;
use num_enum::IntoPrimitive;

pub struct Screen {
    chip_select: Output<'static>,
    data_command: Output<'static>,
    reset: Output<'static>,
    backlight: Output<'static>,
    spi: Spi<'static, Blocking>,
}

impl Screen {
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
        self.chip_select.set_low();

        // Do hardware reset by holding reset low for at least 10us, then wait 5ms for the reset to
        // occur before sending more commands.
        self.reset.set_low();
        Timer::after_micros(100).await;
        self.reset.set_high();
        Timer::after_millis(5).await;

        // Do hardware reset, then wait 120ms for the reset to occur before sending more commands.
        self.send_command(Command::SoftwareReset, &[]);
        Timer::after_millis(120).await;

        // Set portrait orientation
        self.set_orientation(Orientation::Portrait);

        // Set the pixel format to rgb565
        self.send_command(Command::SetPixelFormat, &[0x55]);

        // Have the display emerge from sleep, then wait 5ms for it to wake up
        self.send_command(Command::SleepModeOff, &[]);
        Timer::after_millis(5).await;

        // Turn the display on
        self.send_command(Command::DisplayOn, &[]);

        // Turn on the backlight
        self.set_backlight(true);
    }

    pub fn clear_screen(&mut self, color: u16) {
        const WIDTH: usize = 240;
        const HEIGHT: usize = 320;

        let row = [color; WIDTH];

        self.set_window(0, 0, WIDTH, HEIGHT);
        for i in 0..HEIGHT {
            self.spi.blocking_write(&row).unwrap();
        }
    }

    fn set_window(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
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

    fn send_data(&mut self, data: &[u8]) {
        self.data_command.set_high();
        self.spi.blocking_write(data).unwrap(); // TODO propagate error
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        // TODO set internal width/height
        self.send_command(Command::MemoryAccessControl, &[u8::from(orientation)]);
    }

    pub fn set_backlight(&mut self, on: bool) {
        self.backlight.set_level(on.into());
    }
}

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
    MemoryWrite = 0x2c,
    NormalModeFrameRate = 0xb1,
    SetPageAddress = 0x2b,
    SetPixelFormat = 0x3a,
    SetBrightness = 0x51,
    SleepModeOff = 0x11,
    SleepModeOn = 0x10,
    SoftwareReset = 0x01,
    VerticalScrollAddr = 0x37,
    VerticalScrollDefine = 0x33,
}

#[derive(Copy, Clone, IntoPrimitive)]
#[repr(u8)]
enum Orientation {
    Portrait = 0x40 | 0x08,
    Landscape = 0x20 | 0x08,
    PortraitFlipped = 0x80 | 0x08,
    LandscapeFlipped = 0x40 | 0x80 | 0x20 | 0x08,
}
