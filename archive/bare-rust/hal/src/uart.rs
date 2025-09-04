//! # UART Module
//!
//! This module provides functionality for initializing and using the USART1 peripheral for serial communication.
//! It includes methods for setting up the USART1 with a specified baud rate and sending data through the USART1 interface.
//!
//! ## Functions
//!
//! - `init1`: Initializes the USART1 peripheral with the specified baud rate.
//! - `write1`: Sends a byte of data through the USART1 interface.
//!
//! ## Usage
//!
//! The `init1` function should be called during system startup to configure the USART1 peripheral. The `write1` function
//! can be used to send data through the USART1 interface.
//!
//! ## Example
//!
//! ```rust
//! use crate::hal::*;
//! use crate::hal::clock;
//!
//! fn main() {
//!     clock::init( 16_000_000 );
//!
//!     let tx = gpio::Pin(cpu::GPIOA, 9);
//!     let rx = gpio::Pin(cpu::GPIOA, 10);
//!     let baud_rate: u32 = 115200;
//!     uart::init1(115200,rx,tx);
//!
//!     // Send a byte of data
//!     uart::write1(b'H');
//!     uart::write1(b'i');
//! }
//! ```

use core::ptr;

#[cfg(feature = "std")]
extern crate std;

//use super::board;
use super::cpu;
use super::cpu::*;
use super::gpio;

pub use super::cpu::USART as USART1;
pub use super::cpu::USART as USART2;

#[cfg(feature = "stm32f405")]
pub use super::cpu::DMA as DMA1;

#[cfg(feature = "stm32f405")]
pub use super::cpu::DMA as DMA2;

#[cfg(feature = "stm32f072")]
#[inline(never)]
pub fn init1(baud_rate: u64, tx_pin: gpio::Pin, rx_pin: gpio::Pin) {
    // Enable USART1 & GPIOA clock
    cpu::write!(RCC.apb2enr[USART1EN;1], 1);
    cpu::write!(RCC.ahbenr[IOPAEN;1], 1);

    tx_pin.alt_fun(1, false); // AF1 work for USART1 to 3
    rx_pin.alt_fun(1, false); // AF1 work for USART1 to 3

    // Set baud rate
    let apb_freq: u32 = 48_000_000; // APB clock frequency
    let usart_div: u32 = apb_freq / baud_rate as u32;
    cpu::write!(USART1.brr, usart_div);

    // Enable USART1, transmitter and receiver
    cpu::write!(USART1.cr1[UE;1], 1); // USART enable
    cpu::write!(USART1.cr1[TE;1], 1); // Transmitter enable
    cpu::write!(USART1.cr1[RE;1], 1); // Receiver enable
}

#[cfg(feature = "stm32f072")]
#[inline(never)]
pub fn init2(baud_rate: u64, tx_pin: gpio::Pin, rx_pin: gpio::Pin) {
    // Enable USART2 & GPIOA clock
    cpu::write!(RCC.apb1enr[USART2EN;1], 1);
    cpu::write!(RCC.ahbenr[IOPAEN;1], 1);

    tx_pin.alt_fun(1, false); // AF1 work for USART1 to 3
    rx_pin.alt_fun(1, false); // AF1 work for USART1 to 3

    // Set baud rate
    let apb_freq: u32 = 48_000_000; // APB clock frequency
    let usart_div: u32 = apb_freq / baud_rate as u32;
    cpu::write!(USART2.brr, usart_div);

    cpu::write!( USART2.cr1[M;1], 0); // 8 data bits
    let even_parity = false;
    if even_parity {
        cpu::write!( USART2.cr1[PCE;1], 1); // parity control enable
        cpu::write!( USART2.cr1[PS;1], 0); // even parity
    } else {
        cpu::write!( USART2.cr1[PCE;1], 0); // no parity
    }
    cpu::write!( USART2.cr2[STOP;2], 0b00); // 1 stop bit

    // Enable USART2, transmitter and receiver
    cpu::write!(USART2.cr1[UE;1], 1); // USART enable
    cpu::write!(USART2.cr1[TE;1], 1); // Transmitter enable
    cpu::write!(USART2.cr1[RE;1], 1); // Receiver enable
}

#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn init1(baud_rate: u64, tx_pin: gpio::Pin, rx_pin: gpio::Pin) {
    // enable USART1 & GPIO clock
    cpu::write!( RCC.apb2enr[USART1EN;1], 1);
    cpu::write!( RCC.ahb1enr[GPIOAEN;1], 1);

    tx_pin.alt_fun(7, false); // AF7 work for USART1 to 3
    rx_pin.alt_fun(7, false); // AF7 work for USART1 to 3

    // set baud rate
    // UART 1 is on APB2 bus, which is 84MHz
    let apb_freq: u64 = 84_000_000;
    let div_fixed3: u64 = 1000 * apb_freq / (16 * baud_rate);

    let mantissa: u64 = div_fixed3 / 1000;
    let frac: u64 = (div_fixed3 % 1000) * 16 / 1000;

    cpu::write!( USART1.brr[DIV_Mantissa;11], mantissa as u32);
    cpu::write!( USART1.brr[DIV_Fraction;4], frac as u32);

    cpu::write!( USART1.cr1[M;1], 0); // 8 data bits
    let even_parity = false;
    if even_parity {
        cpu::write!( USART1.cr1[PCE;1], 1); // parity control enable
        cpu::write!( USART1.cr1[PS;1], 0); // even parity
    } else {
        cpu::write!( USART1.cr1[PCE;1], 0); // no parity
    }
    cpu::write!( USART1.cr2[STOP;2], 0b00); // 1 stop bit

    cpu::write!( USART1.cr1[TE;1], 1); // transmit enable
    cpu::write!( USART1.cr1[RE;1], 1); // receive enable
    cpu::write!( USART1.cr1[UE;1], 1); // uart enable
}

#[cfg(feature = "stm32f405")]
pub fn write1(c: u8) {
    #[cfg(not(feature = "std"))]
    while (cpu::read!(USART1.sr[TXE;1]) == 0) {}
    cpu::write!(USART1.dr[DR;8], c as u32);
}

#[cfg(any(feature = "std", not(feature = "stm32f405")))]
pub fn write1_dma(_data: &[u8]) {}

//use core::ptr;

#[cfg(not(feature = "std"))]
#[cfg(feature = "stm32f405")]
pub unsafe fn write1_dma(data: &[u8]) {
    let baud_rate: u32 = 115200;

    // checked
    const USART1_BASE: u32 = 0x40011000;
    const DMA2_BASE: u32 = 0x40026400;
    const RCC_BASE: u32 = 0x40023800;

    // checked
    const USART1_DR: u32 = USART1_BASE + 0x04;
    const USART1_CR1: u32 = USART1_BASE + 0x0C;
    const USART1_CR3: u32 = USART1_BASE + 0x14;
    const USART1_BRR: u32 = USART1_BASE + 0x08;
    //const USART1_SR: u32 = USART1_BASE + 0x00;

    // checked wrong
    const DMA2_S7CR: u32 = DMA2_BASE + 0xb8; //0xA0; // b8
    const DMA2_S7NDTR: u32 = DMA2_BASE + 0xbc; //0xA4; //bc
    const DMA2_S7PAR: u32 = DMA2_BASE + 0xc0; //0xA8; //C0
    const DMA2_S7M0AR: u32 = DMA2_BASE + 0xc4; // 0xAC; //C4
    const DMA2_HISR: u32 = DMA2_BASE + 0x04; // OK
    const DMA2_HIFCR: u32 = DMA2_BASE + 0x0C; // OK

    // checked
    const RCC_AHB1ENR: u32 = RCC_BASE + 0x30;
    const RCC_APB2ENR: u32 = RCC_BASE + 0x44;

    // checked
    const DMA2EN: u32 = 1 << 22;
    const USART1EN: u32 = 1 << 4;

    // checked
    const NVIC_ISER2: u32 = 0xE000E100 + 0x8;
    const DMA2_STREAM7_IRQ: u32 = 70;

    // Enable DMA2 and USART1 clocks
    ptr::write_volatile(RCC_AHB1ENR as *mut u32, DMA2EN);
    ptr::write_volatile(RCC_APB2ENR as *mut u32, USART1EN);

    // Configure USART1
    let apb_freq: u32 = 84_000_000;
    let usart_div: u32 = apb_freq / baud_rate;
    ptr::write_volatile(USART1_BRR as *mut u32, usart_div);
    // odd parity | transmit enable | transmit enable
    ptr::write_volatile(USART1_CR1 as *mut u32, 0x2000 | 0x8 | 0x4); // Enable USART, TX, RX
    ptr::write_volatile(
        USART1_CR3 as *mut u32,
        0x40, // DMAT dma transmition mode
    ); // Enable DMA transmission

    // Configure DMA2 Stream 7
    ptr::write_volatile(DMA2_S7CR as *mut u32, 0); // Disable stream
    while ptr::read_volatile(DMA2_S7CR as *mut u32) & 1 != 0 {} // Wait until disabled

    ptr::write_volatile(DMA2_S7PAR as *mut u32, USART1_DR);
    ptr::write_volatile(DMA2_S7M0AR as *mut u32, data.as_ptr() as u32);
    ptr::write_volatile(DMA2_S7NDTR as *mut u32, data.len() as u32);

    // Configure DMA stream settings
    let dma_cr = (4 << 25) // channel 4
        | (1 << 10) // MINC Memory increment
        | (1 << 8) //  Circular mode enabled
        | (1 << 6) // 0x01 this is memory to pher[herial
        | (1 << 4) // TCIE - transfer complete interupt
        ;
    // Channel 4, memory increment, memory-to-peripheral
    ptr::write_volatile(DMA2_S7CR as *mut u32, dma_cr);

    // Enable DMA2 Stream 7
    ptr::write_volatile(DMA2_S7CR as *mut u32, dma_cr | 1);

    // Enable DMA2 Stream 7 interrupt in NVIC
    ptr::write_volatile(NVIC_ISER2 as *mut u32, 1 << (DMA2_STREAM7_IRQ - 64));

    // Wait for transfer to complete
    while ptr::read_volatile(DMA2_HISR as *mut u32) & (1 << 27) == 0
    // FIEF5 0x40 seems wrong
    {
        if ptr::read_volatile(DMA2_HISR as *mut u32) & (0b1101 << 22) != 0
        // x0B == 11 = 8+2+1 = TEIF4 | DMEIF4 | FEIF4 - seems wrong
        {
            panic!("DMA transfer error");
        }
    }
    // clear transfer complete flag
    ptr::write_volatile(
        DMA2_HIFCR as *mut u32,
        1 << 27, // 0x40 seems wrong
    );
}

#[cfg(not(feature = "std"))]
#[cfg(feature = "stm32f405")]
pub fn write1_dma_save(data: &[u8]) {
    // TODO
    // DMA for USART1_TX
    // Uses DMA 2, Channel 4, Stream 7
    // See page 311 of RM0090

    // Enable DMA2 clocks
    cpu::write!( RCC.ahb1enr[DMA2EN;1], 1);

    cpu::write!(DMA2.s7cr, 0); // Disable DMA2 Stream 7 while configuring
    while cpu::read!(DMA2.s7cr[EN;1]) != 0 {} // Wait until the stream is disabled

    cpu::write!( USART1.cr1[UE;1], 0b0 ); // Disable USART1

    // Enable USART1 DMA transmission
    cpu::write!(USART1.cr3[DMAT;1], 1); // Enable DMA transmission
    cpu::write!(USART1.sr[TC;1], 1); // Clear the transfer complete flag

    cpu::write!( USART1.cr1[UE;1], 0b1 ); // enable USART1

    // Setup DMA transfer from memory to USART1
    let dest: u32 = unsafe { core::ptr::addr_of!((*USART1).dr) as u32 };
    let len: u32 = data.len() as u32;
    let src: u32 = data.as_ptr() as u32;

    assert!(dest == 0x4001_1000 + 0x04); // USART1->DR
    assert!(len == 4);

    cpu::write!(DMA2.s7ndtr, len);
    cpu::write!(DMA2.s7par, dest);
    cpu::write!(DMA2.s7m0ar, src);

    //  Configure DMA channel: memory-to-peripheral, increment memory, enable transfer complete interrupt
    cpu::write!( DMA2.s7cr[CHSEL;2], 0b100); // Using Channel #4

    cpu::write!( DMA2.s7cr[DBM;1], 0b0 ); // Disable double buffer mode

    cpu::write!( DMA2.s7cr[PL;2], 0b10 ); // Set priority level to low

    cpu::write!( DMA2.s7cr[MSIZE;2], 0b00); // Set peripheral size to 1 byte
    cpu::write!( DMA2.s7cr[PSIZE;2], 0b00); // Set peripheral size to 1 byte
    cpu::write!( DMA2.s7cr[PINC;1], 0b0); // Disable peripheral increment mode
    cpu::write!( DMA2.s7cr[MINC;1], 0b1); // Enable memory increment mode

    cpu::write!( DMA2.s7cr[CIRC;1], 0b0); // Disable circular mode
    cpu::write!( DMA2.s7cr[DIR;2], 0b01); // Set direction as memory to peripheral

    cpu::write!( DMA2.s7cr[PFCTRL;1], 0b0); // disable peripheral flow controller

    cpu::write!( DMA2.hifcr[CDMEIF7;1], 0b1); // clear the old direct transfer error flag
    cpu::write!( DMA2.hifcr[CTEIF7;1], 0b1); // clear the old transfer error flag
    cpu::write!( DMA2.hifcr[CFEIF7;1], 0b1); // clear the old FIFO error flag
    cpu::write!( DMA2.hifcr[CTCIF7;1], 0b1); // clear the old transfer complete flag

    cpu::write!( DMA2.s7cr[TCIE;1], 0b1); // Enable transfer complete interrupt
    cpu::write!( DMA2.s7cr[TEIE;1], 0b1); // Enable transfer error interrupt
    cpu::write!( DMA2.s7cr[DMEIE;1], 0b1); // Enable transfer error interrupt

    // DMA2 Stream7 global interrupt = 70 // from SVD file
    // NVIC interrupt enable
    let itr = 70;
    cpu::write!(NVIC.iser[itr / 32], 1 << (itr % 32)); // Enable DMA2 Stream 7 interrupt
    let pri_loc = itr * 4;
    let pri = 0b0011;
    cpu::write!(NVIC.ipr[pri_loc / 32], pri << (pri_loc % 32)); // Enable DMA2 Stream 7 interrupt

    // Clear prior events flags

    // Enable DMA2 Stream 7
    cpu::write!(DMA2.s7cr[EN;1], 1);

    if true {
        // Check if the transfer complete flag is set
        while cpu::read!(DMA2.hisr[TCIF7;1]) == 0 {
            if cpu::read!( DMA2.hisr[TEIF7; 1]) == 0b1 {
                panic!("DMA transfer error interrupt");
            }
            if cpu::read!( DMA2.hisr[FEIF7; 1]) == 0b1 {
                panic!("DMA transfer FIFO error interrupt");
            }
            if cpu::read!( DMA2.hisr[DMEIF7; 1]) == 0b1 {
                panic!("DMA transfer direct mode error interrupt");
            }
        }

        // Clear the transfer complete flag
        cpu::write!(DMA2.hifcr[CTCIF7;1], 1);
    }
}

#[cfg(not(feature = "std"))]
#[cfg(feature = "stm32f405")]
#[inline(never)]
pub fn dma_uart1_irq() {
    if cpu::read!( DMA2.hisr[TEIF7; 1]) == 0b1 {
        panic!("DMA transfer error interrupt");
    }
    if cpu::read!( DMA2.hisr[TCIF7; 1]) == 0b1 {
        // Clear the transfer complete flag
        cpu::write!( DMA2.hifcr[CTCIF7; 1], 0b1);

        // TODO
    } else {
        panic!("DMA transfer complete flag not set");
    }
}

#[cfg(feature = "stm32f072")]
pub fn write1(c: u8) {
    // Wait until transmit data register is empty
    while cpu::read!(USART1.isr[TXE;1]) == 0 {}
    // Write the byte to the data register
    cpu::write!(USART1.tdr, c as u32);
}

#[cfg(feature = "stm32f072")]
pub fn write2(c: u8) {
    // Wait until transmit data register is empty
    while cpu::read!(USART2.isr[TXE;1]) == 0 {}
    // Write the byte to the data register
    cpu::write!(USART2.tdr, c as u32);
}

#[cfg(feature = "stm32f072")]
pub fn empty2() -> bool {
    // Wait until transmit data register is empty
    cpu::read!(USART2.isr[RXNE;1]) == 0
}

#[cfg(feature = "stm32f072")]
pub fn empty1() -> bool {
    // Wait until transmit data register is empty
    cpu::read!(USART1.isr[RXNE;1]) == 0
}

#[cfg(feature = "stm32f405")]
pub fn empty1() -> bool {
    // Wait until transmit data register is empty
    cpu::read!(USART1.sr[RXNE;1]) == 0
}

#[cfg(feature = "stm32f072")]
pub fn read2() -> u8 {
    // Wait until receive data register is empty
    while cpu::read!(USART2.isr[RXNE;1]) == 0 {}
    // Write the byte to the data register
    cpu::read!(USART2.rdr) as u8
}

#[cfg(feature = "stm32f072")]
pub fn read1() -> u8 {
    // Wait until receive data register is empty
    while cpu::read!(USART1.isr[RXNE;1]) == 0 {}
    // Write the byte to the data register
    cpu::read!(USART1.rdr) as u8
}

#[cfg(feature = "stm32f405")]
pub fn read1() -> u8 {
    // Wait until receive data register is empty
    while cpu::read!(USART1.sr[RXNE;1]) == 0 {}
    // Write the byte to the data register
    cpu::read!(USART1.dr) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock;

    #[test]
    fn test_uart() {
        clock::init(16_000_000);

        let tx = gpio::Pin(cpu::GPIOA, 9);
        let rx = gpio::Pin(cpu::GPIOA, 10);
        let baud_rate: u64 = 115200;

        init1(baud_rate, tx, rx);
        write1(b'O');
        write1(b'K');
    }
}
