pub mod led;

use led::Led;
use stm32f0xx_hal::{
    pac::{self, Interrupt},
    prelude::*,
    serial::Serial,
    timers::{Event, Timer},
};

mod led_a {
    use stm32f0xx_hal::gpio::{gpioa, Output, PushPull};
    pub type RedPin = gpioa::PA4<Output<PushPull>>;
    pub type GreenPin = gpioa::PA6<Output<PushPull>>;
    pub type BluePin = gpioa::PA7<Output<PushPull>>;
}

mod led_b {
    use stm32f0xx_hal::gpio::{gpiob, Output, PushPull};
    pub type RedPin = gpiob::PB0<Output<PushPull>>;
    pub type GreenPin = gpiob::PB14<Output<PushPull>>;
    pub type BluePin = gpiob::PB15<Output<PushPull>>;
}

mod console {
    use stm32f0xx_hal::{
        gpio::{gpioa, Alternate, AF1},
        pac,
    };
    pub type Tx = gpioa::PA9<Alternate<AF1>>;
    pub type Rx = gpioa::PA10<Alternate<AF1>>;
    pub type Usart = pac::USART1;
    pub const BAUD_RATE: u32 = 115_200;
}

mod timer {
    use stm32f0xx_hal::pac;
    pub type Timer = pac::TIM7;
}

pub type LedA = Led<led_a::RedPin, led_a::GreenPin, led_a::BluePin, true>;
pub type LedB = Led<led_b::RedPin, led_b::GreenPin, led_b::BluePin, true>;
pub type Console = Serial<console::Usart, console::Tx, console::Rx>;
pub type Timer1Hz = Timer<timer::Timer>;

pub struct Board {
    pub led_a: LedA,
    pub led_b: LedB,
    pub console: Console,
    pub timer: Timer1Hz,
}

impl Board {
    pub fn new() -> Self {
        let cp = cortex_m::peripheral::Peripherals::take().unwrap();
        let mut dp = pac::Peripherals::take().unwrap();
        let mut rcc = dp
            .RCC
            .configure()
            .hsi48()
            .enable_crs(dp.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut dp.FLASH);
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);

        // Configure GPIO pins
        let (led_a_r, led_a_g, led_a_b, led_b_r, led_b_g, led_b_b, tx_pin, rx_pin) =
            cortex_m::interrupt::free(|cs| {
                let led_a_r = gpioa.pa4.into_push_pull_output(cs);
                let led_a_g = gpioa.pa6.into_push_pull_output(cs);
                let led_a_b = gpioa.pa7.into_push_pull_output(cs);
                let led_b_r = gpiob.pb0.into_push_pull_output(cs);
                let led_b_g = gpiob.pb14.into_push_pull_output(cs);
                let led_b_b = gpiob.pb15.into_push_pull_output(cs);
                let tx_pin = gpioa.pa9.into_alternate_af1(cs);
                let rx_pin = gpioa.pa10.into_alternate_af1(cs);
                (
                    led_a_r, led_a_g, led_a_b, led_b_r, led_b_g, led_b_b, tx_pin, rx_pin,
                )
            });

        // Configure LEDs
        let led_a = Led::new(led_a_r, led_a_g, led_a_b);
        let led_b = Led::new(led_b_r, led_b_g, led_b_b);

        // Configure the console UART
        let console = Serial::usart1(
            dp.USART1,
            (tx_pin, rx_pin),
            console::BAUD_RATE.bps(),
            &mut rcc,
        );

        // Configure the 1s timer
        let mut timer = Timer::tim7(dp.TIM7, 1.hz(), &mut rcc);
        timer.listen(Event::TimeOut);

        let mut nvic = cp.NVIC;
        unsafe {
            nvic.set_priority(Interrupt::TIM7, 1);
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM7);
        }
        cortex_m::peripheral::NVIC::unpend(Interrupt::TIM7);

        Self {
            led_a,
            led_b,
            console,
            timer,
        }
    }
}
