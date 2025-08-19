#![no_std]
#![no_main]

mod ev12;

use ev12 as board;

use board::*;
use ui_app::*;

use panic_halt as _;

use core::cell::RefCell;
use core::fmt::Write;
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use heapless::mpmc::Q64;
use stm32f4xx_hal::{
    gpio::{PinState, Pull, Speed},
    interrupt,
    pac::{self, Interrupt},
    prelude::*,
    serial::{Config as SerialConfig, Serial},
};

static PTT_BUTTON: Mutex<RefCell<Option<PttButton>>> = Mutex::new(RefCell::new(None));
static AI_BUTTON: Mutex<RefCell<Option<AiButton>>> = Mutex::new(RefCell::new(None));

static EVENT_QUEUE: Q64<Event> = Q64::new();

#[entry]
fn main() -> ! {
    /*
    let mut board = Board::new();
    let mut app = App::start(&mut board);

    // Make buttons accessible to interrupts
    free(|cs| {
        PTT_BUTTON.borrow(cs).replace(board.ptt_button.take());
        AI_BUTTON.borrow(cs).replace(board.ai_button.take());
    });

    // Enable the EXTI interrupts
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0);
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI1);
    }

    // Feed events to the app
    loop {
        let Some(event) = EVENT_QUEUE.dequeue() else {
            continue;
        };

        app.handle(event, &mut board);
    }
    */

    let mut dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();

    // https://docs.rs/stm32f4xx-hal/latest/stm32f4xx_hal/rcc/index.html
    let rcc = dp.RCC.constrain();
    let mut clocks = rcc
        .cfgr
        .use_hse(6.MHz())
        .bypass_hse_oscillator()
        .sysclk(72.MHz())
        //.pclk1(24.MHz())
        //.require_pll48clk()
        .freeze();

    // Manual clock setup, cloned from bare-rust
    /*
    let rcc = unsafe { &*pac::RCC::ptr() };
    rcc.cr().modify(|_, w| w.hseon().set_bit());
    while rcc.cr().read().hserdy().bit_is_clear() {}

    rcc.cfgr().modify(|_, w| w.pllsrc().set_bits(1));
    */

    //////////

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();

    // Configure the status LEDs
    let mut r = gpioa.pa6.into_push_pull_output_in_state(PinState::High);
    let mut g = gpioc.pc5.into_push_pull_output_in_state(PinState::High);
    let mut b = gpioa.pa1.into_push_pull_output_in_state(PinState::High);

    // Configure UART1 pins according to the reference configuration:
    // PA9 (TX) and PA10 (RX) with alternate function 7 (USART1)
    let mut uart_tx_pin = gpioa.pa9.into_alternate::<7>();
    uart_tx_pin.set_internal_resistor(Pull::None);
    uart_tx_pin.set_speed(Speed::VeryHigh);

    let mut uart_rx_pin = gpioa.pa10.into_alternate::<7>();
    uart_rx_pin.set_internal_resistor(Pull::None);
    uart_rx_pin.set_speed(Speed::VeryHigh);

    // Configure UART1 with standard parameters
    let serial_config = SerialConfig::default()
        .baudrate(115200.bps())
        .wordlength_9()
        .parity_even()
        .stopbits(stm32f4xx_hal::serial::config::StopBits::STOP1);

    // Create the serial interface
    let mut serial: Serial<_, u8> = Serial::new(
        dp.USART1,
        (uart_tx_pin, uart_rx_pin),
        serial_config,
        &clocks,
    )
    .unwrap();

    // Create a timer-based delay
    let mut delay = dp.TIM5.delay_us(&mut clocks);

    //////////

    r.set_low();

    loop {
        serial.write_str("hello world");
        r.toggle();
        delay.delay_ms(1000);
    }
}

#[interrupt]
fn EXTI1() {
    static mut BUTTON: Option<PttButton> = None;

    let button =
        BUTTON.get_or_insert_with(|| free(|cs| PTT_BUTTON.borrow(cs).replace(None).unwrap()));

    let _ = button.clear_interrupt_pending_bit();

    let event = if button.is_low() {
        Event::PttDown
    } else {
        Event::PttUp
    };

    let _ = EVENT_QUEUE.enqueue(event);
}

#[interrupt]
fn EXTI0() {
    static mut BUTTON: Option<AiButton> = None;

    let button =
        BUTTON.get_or_insert_with(|| free(|cs| AI_BUTTON.borrow(cs).replace(None).unwrap()));

    let _ = button.clear_interrupt_pending_bit();

    let event = if button.is_low() {
        Event::AiDown
    } else {
        Event::AiUp
    };

    let _ = EVENT_QUEUE.enqueue(event);
}
