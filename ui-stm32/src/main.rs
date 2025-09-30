#![no_std]
#![no_main]
#![allow(dead_code)] // XXX

mod board;
mod hal_i2s;
use hal_i2s::*;

use board::{AudioControl, Button, Keyboard, NetRx};
use ui_app::Button as ButtonId;
use ui_app::Event;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{mode::Async, usart::UartRx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Configuration parameters
const EVENT_QUEUE_DEPTH: usize = 10;
const KEYBOARD_SCAN_MILLIS: u64 = 50;

type EventChannel = Channel<CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;
type EventSender = Sender<'static, CriticalSectionRawMutex, Event, EVENT_QUEUE_DEPTH>;

static EVENT_QUEUE: EventChannel = Channel::new();

#[embassy_executor::task(pool_size = 2)]
async fn monitor_button(mut button: Button, id: ButtonId, events: EventSender) {
    loop {
        button.wait_for_rising_edge().await;
        events.send(Event::ButtonDown(id)).await;
        button.wait_for_falling_edge().await;
        events.send(Event::ButtonUp(id)).await;
    }
}

#[embassy_executor::task]
async fn monitor_keyboard(mut keyboard: Keyboard, events: EventSender) {
    loop {
        let _ = Timer::after_millis(KEYBOARD_SCAN_MILLIS).await;
        for event in keyboard.scan() {
            events.send(event).await;
        }
    }
}

#[embassy_executor::task]
async fn monitor_net(from: UartRx<'static, Async>, events: EventSender) {
    const DMA_BUFFER_SIZE: usize = 1024;

    // Wrap the raw receiver in a DMA-buffered, SLIP-parsing, TLV-parsing version
    let mut dma_buf = [0u8; DMA_BUFFER_SIZE];
    let mut from = from.into_ring_buffered(&mut dma_buf);
    let mut from = NetRx::new(&mut from);

    loop {
        let Some(from_net) = from.next().await else {
            continue;
        };

        events.send(Event::FromNet(from_net)).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    /*
    info!("about to instantiate board");

    let mut board = Board::new().await;
    let mut app = App::new();

    info!("done setting up board and app");

    // Capture button events
    unwrap!(spawner.spawn(monitor_button(
        board.button_a.take().unwrap(),
        ButtonId::A,
        EVENT_QUEUE.sender()
    )));

    unwrap!(spawner.spawn(monitor_button(
        board.button_b.take().unwrap(),
        ButtonId::B,
        EVENT_QUEUE.sender()
    )));

    // Capture keyboard events
    unwrap!(spawner.spawn(monitor_keyboard(
        board.keyboard.take().unwrap(),
        EVENT_QUEUE.sender()
    )));

    // Capture UART events from the NET chip
    unwrap!(spawner.spawn(monitor_net(
        board.net_rx.take().unwrap(),
        EVENT_QUEUE.sender()
    )));

    debug!("app start");
    app.start(&mut board);

    // Main event loop
    loop {
        let event = EVENT_QUEUE.receive().await;
        app.handle(event, &mut board);
    }
    */

    /*
    ///// EEPROM Demo /////

    // Read the current contents of the EEPROM
    const I2C_ADDR: u8 = 0x50;
    const ADDR: u8 = 0x00;

    let mut data = [0u8; 256];
    board
        .i2c
        .blocking_write_read(I2C_ADDR, &[ADDR], &mut data)
        .unwrap();
    let hex: heapless::String<1024> = data.encode_hex();
    info!("eeprom before {}", hex);

    // Overwrite the EEPROM with a new value
    let mut data = [0xA0; 17];
    for i in (0_u8..=0xff).step_by(16) {
        data[0] = i;
        board.i2c.blocking_write(I2C_ADDR, &data).unwrap();
        Timer::after_millis(10).await;
    }

    // Read the value back out of the EEPROM
    let mut data = [0u8; 256];
    board
        .i2c
        .blocking_write_read(I2C_ADDR, &[ADDR], &mut data)
        .unwrap();
    let hex: heapless::String<1024> = data.encode_hex();
    info!("eeprom after {}", hex);
    */

    ///// Audio Chip /////
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

    // Do audio chip setup over I2C
    let config = {
        use embassy_stm32::{gpio::Speed, i2c::*, time::Hertz};

        let mut config = Config::default();

        config.frequency = Hertz(100_000);
        config.gpio_speed = Speed::VeryHigh;
        config.sda_pullup = false;
        config.scl_pullup = false;
        config.timeout = embassy_time::Duration::from_millis(1000);

        config
    };
    let i2c = embassy_stm32::i2c::I2c::new_blocking(p.I2C1, p.PB6, p.PB7, config);
    let mut audio_control = AudioControl::new(i2c);
    audio_control.init().await;

    // Start the SysTick timer
    hal_init_tick(168_000_000);

    // HAL_I2S_MspInit() - Configure I2S3 GPIO and clocks
    hal_i2s_msp_init();

    // MX_I2S3_Init() - Configure I2S3 parameters
    let mut i2s = I2sHandle::new_spi3();

    i2s.init.mode = I2S_MODE_SLAVE_TX;
    i2s.init.standard = I2S_STANDARD_PHILIPS;
    i2s.init.data_format = I2S_DATAFORMAT_16B_EXTENDED;
    i2s.init.mclk_output = I2S_MCLKOUTPUT_DISABLE;
    i2s.init.audio_freq = I2S_AUDIOFREQ_8K;
    i2s.init.cpol = I2S_CPOL_LOW;
    i2s.init.clock_source = I2S_CLOCKSOURCE_PLLI2S;
    i2s.init.full_duplex_mode = I2S_FULLDUPLEXMODE_ENABLE;

    let rv = hal_i2s_init(&mut i2s);
    if rv != HalStatus::Ok {
        defmt::panic!("Failed to initialize I2S: {}", rv);
    }

    trace!("Ready to roll 😎");

    let square_wave: [u16; 36] = [
        0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff,
        0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x1fff, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000,
    ];
    let square_frame: [u16; 16_000] = core::array::from_fn(|i| square_wave[i % square_wave.len()]);

    trace!("before tx");
    let rv = hal_i2s_transmit(&mut i2s, &square_frame, 100);
    if rv != HalStatus::Ok {
        defmt::panic!("Failed to transmit: {} {}", rv, i2s.error_code);
    }
    trace!("after tx");

    /*
    trace!("before txrx");
    let mut last_frame = [0; 16_000];
    let mut curr_frame = [0; 16_000];
    loop {
        let rv = hal_i2sex_transmit_receive(&mut i2s, &last_frame, &mut curr_frame, 100);
        if rv != HalStatus::Ok {
            defmt::panic!("Failed to transmit: {} {}", rv, i2s.error_code);
        }

        defmt::trace!("raw: {:?}", curr_frame);

        let total_energy = curr_frame.iter().map(|x| *x as u32).sum::<u32>();
        let avg_energy = total_energy / 16000;
        let max_energy = *curr_frame.iter().max().unwrap();
        defmt::trace!("energy: {:?} {:?} {}", total_energy, avg_energy, max_energy);

        last_frame.copy_from_slice(&curr_frame);

        // Amplify noise to make it audible
        //
        //let amp = 0x1fff / max_energy as u16;
        //last_frame.iter_mut().for_each(|x| *x *= amp);
    }
    trace!("after txrx");
    */
}

/// HAL_I2S_MspInit - I2S3 Hardware Initialization
///
/// Direct translation of HAL_I2S_MspInit() from stm32f4xx_hal_msp.c
/// Configures:
/// - PLLI2S clock (N=50, R=2)
/// - SPI3 peripheral clock
/// - GPIO pins for I2S3 full duplex operation using direct register access
/// - Skips DMA initialization as requested
fn hal_i2s_msp_init() {
    use core::ptr;

    // STM32F4 Register base addresses
    const RCC_BASE: u32 = 0x40023800;
    const GPIOA_BASE: u32 = 0x40020000;
    const GPIOB_BASE: u32 = 0x40020400;
    const GPIOC_BASE: u32 = 0x40020800;

    // RCC registers
    const RCC_CR: u32 = RCC_BASE + 0x00; // RCC Clock Control Register
    const RCC_PLLI2SCFGR: u32 = RCC_BASE + 0x84; // RCC PLLI2S Configuration Register
    const RCC_APB1ENR: u32 = RCC_BASE + 0x40; // RCC APB1 Peripheral Clock Enable Register
    const RCC_AHB1ENR: u32 = RCC_BASE + 0x30; // RCC AHB1 Peripheral Clock Enable Register

    // GPIO register offsets
    const GPIO_MODER_OFFSET: u32 = 0x00; // GPIO Port mode register
    const GPIO_OTYPER_OFFSET: u32 = 0x04; // GPIO Port output type register
    const GPIO_OSPEEDR_OFFSET: u32 = 0x08; // GPIO Port output speed register
    const GPIO_PUPDR_OFFSET: u32 = 0x0C; // GPIO Port pull-up/pull-down register
    const GPIO_AFRH_OFFSET: u32 = 0x24; // GPIO Alternate function high register
    const GPIO_AFRL_OFFSET: u32 = 0x20; // GPIO Alternate function low register

    // RCC bit definitions
    const RCC_CR_PLLI2SON: u32 = 1 << 26; // PLLI2S Enable
    const RCC_CR_PLLI2SRDY: u32 = 1 << 27; // PLLI2S Ready flag
    const RCC_APB1ENR_SPI3EN: u32 = 1 << 15; // SPI3 clock enable
    const RCC_AHB1ENR_GPIOAEN: u32 = 1 << 0; // GPIOA clock enable
    const RCC_AHB1ENR_GPIOBEN: u32 = 1 << 1; // GPIOB clock enable
    const RCC_AHB1ENR_GPIOCEN: u32 = 1 << 2; // GPIOC clock enable

    unsafe {
        // Configure PLLI2S Clock (PeriphClkInitStruct from C HAL)
        let mut cr = ptr::read_volatile(RCC_CR as *const u32);
        if (cr & RCC_CR_PLLI2SON) != 0 {
            // Disable PLLI2S first
            cr &= !RCC_CR_PLLI2SON;
            ptr::write_volatile(RCC_CR as *mut u32, cr);
            // Wait for PLLI2S to be disabled
            while {
                cr = ptr::read_volatile(RCC_CR as *const u32);
                (cr & RCC_CR_PLLI2SRDY) != 0
            } {}
        }

        // Configure PLLI2S: N=50, R=2 (matching C HAL)
        let plli2s_n = 50 << 6; // PLLI2SN = 50
        let plli2s_r = 0 << 28; // PLLI2SR = 2 (encoded as 0)
        ptr::write_volatile(RCC_PLLI2SCFGR as *mut u32, plli2s_n | plli2s_r);

        // Enable PLLI2S
        cr = ptr::read_volatile(RCC_CR as *const u32);
        ptr::write_volatile(RCC_CR as *mut u32, cr | RCC_CR_PLLI2SON);

        // Wait for PLLI2S ready
        while {
            cr = ptr::read_volatile(RCC_CR as *const u32);
            (cr & RCC_CR_PLLI2SRDY) == 0
        } {}

        // Enable peripheral clocks (from C HAL)
        let apb1enr = ptr::read_volatile(RCC_APB1ENR as *const u32);
        ptr::write_volatile(RCC_APB1ENR as *mut u32, apb1enr | RCC_APB1ENR_SPI3EN);

        // Enable GPIO clocks
        let ahb1enr = ptr::read_volatile(RCC_AHB1ENR as *const u32);
        ptr::write_volatile(
            RCC_AHB1ENR as *mut u32,
            ahb1enr | RCC_AHB1ENR_GPIOAEN | RCC_AHB1ENR_GPIOBEN | RCC_AHB1ENR_GPIOCEN,
        );

        // Configure GPIO pins (direct translation from C HAL GPIO_InitStruct)

        // PA15 -> I2S3_WS (AF6, Push-Pull, No Pull, Low Speed)
        let mut moder = ptr::read_volatile((GPIOA_BASE + GPIO_MODER_OFFSET) as *const u32);
        moder &= !(0x3 << (15 * 2)); // Clear PA15 mode bits
        moder |= 0x2 << (15 * 2); // Set PA15 to alternate function mode
        ptr::write_volatile((GPIOA_BASE + GPIO_MODER_OFFSET) as *mut u32, moder);

        let mut otyper = ptr::read_volatile((GPIOA_BASE + GPIO_OTYPER_OFFSET) as *const u32);
        otyper &= !(1 << 15); // PA15 push-pull
        ptr::write_volatile((GPIOA_BASE + GPIO_OTYPER_OFFSET) as *mut u32, otyper);

        let mut pupdr = ptr::read_volatile((GPIOA_BASE + GPIO_PUPDR_OFFSET) as *const u32);
        pupdr &= !(0x3 << (15 * 2)); // No pull-up/pull-down
        ptr::write_volatile((GPIOA_BASE + GPIO_PUPDR_OFFSET) as *mut u32, pupdr);

        let mut ospeedr = ptr::read_volatile((GPIOA_BASE + GPIO_OSPEEDR_OFFSET) as *const u32);
        ospeedr &= !(0x3 << (15 * 2)); // PA15 low speed
        ptr::write_volatile((GPIOA_BASE + GPIO_OSPEEDR_OFFSET) as *mut u32, ospeedr);

        let mut afrh = ptr::read_volatile((GPIOA_BASE + GPIO_AFRH_OFFSET) as *const u32);
        afrh &= !(0xF << ((15 - 8) * 4)); // Clear PA15 AF
        afrh |= 6 << ((15 - 8) * 4); // AF6 for PA15
        ptr::write_volatile((GPIOA_BASE + GPIO_AFRH_OFFSET) as *mut u32, afrh);

        // PC10 -> I2S3_CK (AF6, Push-Pull, No Pull, Low Speed)
        moder = ptr::read_volatile((GPIOC_BASE + GPIO_MODER_OFFSET) as *const u32);
        moder &= !(0x3 << (10 * 2));
        moder |= 0x2 << (10 * 2);
        ptr::write_volatile((GPIOC_BASE + GPIO_MODER_OFFSET) as *mut u32, moder);

        otyper = ptr::read_volatile((GPIOC_BASE + GPIO_OTYPER_OFFSET) as *const u32);
        otyper &= !(1 << 10);
        ptr::write_volatile((GPIOC_BASE + GPIO_OTYPER_OFFSET) as *mut u32, otyper);

        pupdr = ptr::read_volatile((GPIOC_BASE + GPIO_PUPDR_OFFSET) as *const u32);
        pupdr &= !(0x3 << (10 * 2));
        ptr::write_volatile((GPIOC_BASE + GPIO_PUPDR_OFFSET) as *mut u32, pupdr);

        ospeedr = ptr::read_volatile((GPIOC_BASE + GPIO_OSPEEDR_OFFSET) as *const u32);
        ospeedr &= !(0x3 << (10 * 2));
        ptr::write_volatile((GPIOC_BASE + GPIO_OSPEEDR_OFFSET) as *mut u32, ospeedr);

        afrh = ptr::read_volatile((GPIOC_BASE + GPIO_AFRH_OFFSET) as *const u32);
        afrh &= !(0xF << ((10 - 8) * 4));
        afrh |= 6 << ((10 - 8) * 4); // AF6 for PC10
        ptr::write_volatile((GPIOC_BASE + GPIO_AFRH_OFFSET) as *mut u32, afrh);

        // PB4 -> I2S3_ext_SD (AF7, Push-Pull, No Pull, Low Speed)
        moder = ptr::read_volatile((GPIOB_BASE + GPIO_MODER_OFFSET) as *const u32);
        moder &= !(0x3 << (4 * 2));
        moder |= 0x2 << (4 * 2);
        ptr::write_volatile((GPIOB_BASE + GPIO_MODER_OFFSET) as *mut u32, moder);

        otyper = ptr::read_volatile((GPIOB_BASE + GPIO_OTYPER_OFFSET) as *const u32);
        otyper &= !(1 << 4);
        ptr::write_volatile((GPIOB_BASE + GPIO_OTYPER_OFFSET) as *mut u32, otyper);

        pupdr = ptr::read_volatile((GPIOB_BASE + GPIO_PUPDR_OFFSET) as *const u32);
        pupdr &= !(0x3 << (4 * 2));
        ptr::write_volatile((GPIOB_BASE + GPIO_PUPDR_OFFSET) as *mut u32, pupdr);

        ospeedr = ptr::read_volatile((GPIOB_BASE + GPIO_OSPEEDR_OFFSET) as *const u32);
        ospeedr &= !(0x3 << (4 * 2));
        ptr::write_volatile((GPIOB_BASE + GPIO_OSPEEDR_OFFSET) as *mut u32, ospeedr);

        let mut afrl = ptr::read_volatile((GPIOB_BASE + GPIO_AFRL_OFFSET) as *const u32);
        afrl &= !(0xF << (4 * 4));
        afrl |= 7 << (4 * 4); // AF7 for PB4 (I2S3ext)
        ptr::write_volatile((GPIOB_BASE + GPIO_AFRL_OFFSET) as *mut u32, afrl);

        // PB5 -> I2S3_SD (AF6, Push-Pull, No Pull, Low Speed)
        moder = ptr::read_volatile((GPIOB_BASE + GPIO_MODER_OFFSET) as *const u32);
        moder &= !(0x3 << (5 * 2));
        moder |= 0x2 << (5 * 2);
        ptr::write_volatile((GPIOB_BASE + GPIO_MODER_OFFSET) as *mut u32, moder);

        otyper = ptr::read_volatile((GPIOB_BASE + GPIO_OTYPER_OFFSET) as *const u32);
        otyper &= !(1 << 5);
        ptr::write_volatile((GPIOB_BASE + GPIO_OTYPER_OFFSET) as *mut u32, otyper);

        pupdr = ptr::read_volatile((GPIOB_BASE + GPIO_PUPDR_OFFSET) as *const u32);
        pupdr &= !(0x3 << (5 * 2));
        ptr::write_volatile((GPIOB_BASE + GPIO_PUPDR_OFFSET) as *mut u32, pupdr);

        ospeedr = ptr::read_volatile((GPIOB_BASE + GPIO_OSPEEDR_OFFSET) as *const u32);
        ospeedr &= !(0x3 << (5 * 2));
        ptr::write_volatile((GPIOB_BASE + GPIO_OSPEEDR_OFFSET) as *mut u32, ospeedr);

        afrl = ptr::read_volatile((GPIOB_BASE + GPIO_AFRL_OFFSET) as *const u32);
        afrl &= !(0xF << (5 * 4));
        afrl |= 6 << (5 * 4); // AF6 for PB5
        ptr::write_volatile((GPIOB_BASE + GPIO_AFRL_OFFSET) as *mut u32, afrl);
    }

    // Note: DMA initialization skipped as requested
    // Note: I2S3 interrupt configuration skipped (HAL_NVIC_SetPriority/EnableIRQ)
}

#[cortex_m_rt::exception]
fn SysTick() {
    hal_inc_tick();
}
