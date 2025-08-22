#![no_std]
#![no_main]

mod ev12;

use ev12 as board;

use board::*;
use ui_app::*;

use panic_halt as _;

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use cortex_m::interrupt::{free, Mutex};
use cortex_m_rt::entry;
use heapless::mpmc::Q64;
use stm32f4xx_hal::{
    dma::{StreamX, StreamsTuple},
    gpio::{self, Output, PinState, Pull, Speed},
    interrupt,
    pac::{self, Interrupt, DMA2, USART1},
    prelude::*,
    serial::{
        dma::{RxDMA, SerialDma, TxDMA},
        Config as SerialConfig, Serial,
    },
};

static PTT_BUTTON: Mutex<RefCell<Option<PttButton>>> = Mutex::new(RefCell::new(None));
static AI_BUTTON: Mutex<RefCell<Option<AiButton>>> = Mutex::new(RefCell::new(None));

type MgmtSerial =
    SerialDma<USART1, TxDMA<USART1, StreamX<DMA2, 7>, 4>, RxDMA<USART1, StreamX<DMA2, 5>, 4>>;
static MGMT_SERIAL: Mutex<RefCell<Option<MgmtSerial>>> = Mutex::new(RefCell::new(None));
static DONE: AtomicBool = AtomicBool::new(false);

static BLUE_LED: Mutex<RefCell<Option<gpio::PA1<Output>>>> = Mutex::new(RefCell::new(None));
static GREEN_LED: Mutex<RefCell<Option<gpio::PC5<Output>>>> = Mutex::new(RefCell::new(None));

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
        .freeze();

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

    // Set up DMA
    let dma2 = StreamsTuple::new(dp.DMA2);

    let mgmt_serial = serial.use_dma(dma2.7, dma2.5);
    cortex_m::interrupt::free(|cs| {
        GREEN_LED.borrow(cs).borrow_mut().replace(g);
        BLUE_LED.borrow(cs).borrow_mut().replace(b);
        MGMT_SERIAL.borrow(cs).borrow_mut().replace(mgmt_serial);
    });

    // Create a timer-based delay
    let mut delay = dp.TIM5.delay_us(&mut clocks);

    // Enable the required interrupts
    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0);
        cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI1);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA1_STREAM5);
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::DMA1_STREAM7);
    }

    r.set_low();

    loop {
        cortex_m::interrupt::free(|cs| unsafe {
            if let Some(mgmt_serial) = MGMT_SERIAL.borrow(cs).borrow_mut().as_mut() {
                let _ = mgmt_serial.write(b"hello world\n");
            }
        });

        r.toggle();
        delay.delay_ms(1000);

        cortex_m::interrupt::free(|cs| unsafe {
            if let Some(mgmt_serial) = MGMT_SERIAL.borrow(cs).borrow_mut().as_mut() {
                let _ = mgmt_serial.write_dma(b"hello motherfucker!\n", None);
            }
        });

        //while !DONE.load(Ordering::SeqCst) {}
        //DONE.store(false, Ordering::SeqCst);

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

#[interrupt]
fn USART2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(g) = GREEN_LED.borrow(cs).borrow_mut().as_mut() {
            g.toggle();
        }

        if let Some(usart2_dma) = MGMT_SERIAL.borrow(cs).borrow_mut().as_mut() {
            usart2_dma.handle_error_interrupt();
        }
    });
}

#[interrupt]
fn DMA1_STREAM5() {
    cortex_m::interrupt::free(|cs| {
        if let Some(b) = BLUE_LED.borrow(cs).borrow_mut().as_mut() {
            b.toggle();
        }

        if let Some(usart2_dma) = MGMT_SERIAL.borrow(cs).borrow_mut().as_mut() {
            usart2_dma.handle_dma_interrupt();
            DONE.store(true, Ordering::SeqCst);
        }
    });
}

#[interrupt]
fn DMA1_STREAM7() {
    cortex_m::interrupt::free(|cs| {
        if let Some(b) = BLUE_LED.borrow(cs).borrow_mut().as_mut() {
            b.toggle();
        }

        if let Some(usart2_dma) = MGMT_SERIAL.borrow(cs).borrow_mut().as_mut() {
            usart2_dma.handle_dma_interrupt();
            DONE.store(true, Ordering::SeqCst);
        }
    });
}
