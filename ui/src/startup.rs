//! Startup code for ARM Cortex-M microcontrollers
//!
//! The `Reset_Handler` function is the entry point of the program
//! and is called after the microcontroller is reset. It initializes
//! the `.bss` and `.data` sections, and then calls the `main` function.
//!
//! The `Default_Handler` function is an exception handler that is called
//! when an exception with no specific handler is raised. It simply
//! turns on the red LED and enters an infinite loop.
//!
//! The `XXX_IRQHandler` functions are interrupt handlers that are called
//! when a specific interrupt is raised.
//!
//! More information about the startup process can be found in
//! the [Cortex-M4 Technical Reference Manual](https://documentation-service.arm.com/static/5f19da2a20b7cf4bc524d99a).
//!
//! The code in this module is closely tied to the linker configuration
//! script in the `linker.ld` file.
//!

#[cfg(not(feature = "std"))]
use crate::semihost;

#[cfg(not(feature = "std"))]
use core::ptr;

#[cfg(not(feature = "std"))]
use super::stack::STACK_PAINT_BYTE;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use bsp::led;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use bsp::led::Color;

#[cfg(not(feature = "std"))]
extern "C" {
    fn main() -> !;
}

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use core::panic::PanicInfo;

#[cfg(target_arch = "arm")]
#[cfg(not(feature = "std"))]
#[inline(never)]
#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    led::set(Color::Red);
    loop {
        #[cfg(feature = "exit")]
        hal::semihost::exit(0);
    }
}

extern "C" {
    static mut _sbss: u8;
    static _ebss: u8;
    static mut _sdata: u8;
    static _edata: u8;
    static _sidata: u8;
    static _estack: u8;
    static mut _heap_start: u8;
}

#[cfg(not(feature = "std"))]
#[no_mangle]
#[inline(never)]
#[allow(static_mut_refs)]
pub extern "C" fn Reset_Handler() -> ! {
    unsafe {
        // initialize the BSS section with zeros
        let count = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
        ptr::write_bytes(&mut _sbss as *mut u8, 0, count);
    }
    unsafe {
        // initialize the DATA section with the data from the flash
        let count = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
        ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count);
    }
    unsafe {
        // initialize the heap and stack to 0xC1
        // leave 100 bytes free for this function
        let count = &_estack as *const u8 as usize - &_heap_start as *const u8 as usize - 100;
        ptr::write_bytes(&mut _heap_start as *mut u8, STACK_PAINT_BYTE, count);
    }

    unsafe { main() }
}

#[cfg(target_arch = "arm")]
#[cfg(not(feature = "std"))]
#[inline(never)]
#[no_mangle]
pub extern "C" fn Default_Handler() {
    led::set(Color::Red);
    semihost::exit_no_status();
    #[allow(unreachable_code)]
    loop {
        #[cfg(feature = "exit")]
        hal::semihost::exit(0);
    }
}

#[cfg(feature = "std")]
#[no_mangle]
pub extern "C" fn Default_Handler() {
    #[cfg(not(test))]
    panic!("Default_Handler should never be called in simulation");
}

#[cfg(target_arch = "arm")]
#[cfg(not(feature = "std"))]
#[inline(never)]
#[no_mangle]
pub extern "C" fn Dma_Uart1_Handler() {
    led::set(Color::White); // TODO: remove this
    hal::uart::dma_uart1_irq();
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_HandlerA() {
    Default_Handler();
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_HandlerB() {
    Default_Handler();
}

#[cfg(not(feature = "std"))]
#[no_mangle]

pub extern "C" fn Default_HandlerC() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]

pub extern "C" fn Default_HandlerD() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]

pub extern "C" fn Default_HandlerE() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]

pub extern "C" fn Default_HandlerF() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_HandlerG() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_HandlerH() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_Handler1() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_Handler2() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_Handler3() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_Handler4() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_Handler5() {
    Default_Handler();
}
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn Default_Handler6() {
    Default_Handler();
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn TIM1_UP_TIM10_IRQHandler() {
    //hal::timer::handle_tim1_irq();
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn TIM2_IRQHandler() {
    hal::timer::handle_tim2_irq();
}

#[allow(dead_code)]
pub union IrqVector {
    not_used: u32,
    handler: unsafe extern "C" fn(),
}

#[cfg(target_arch = "arm")]
#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static Reset_Vector: extern "C" fn() -> ! = Reset_Handler;

#[cfg(target_arch = "arm")]
#[link_section = ".vector_table.exceptions"]
#[no_mangle]
pub static Exceptions: [IrqVector; 14 + 82] = [
    IrqVector {
        handler: Default_HandlerA,
    }, // NMI
    IrqVector {
        handler: Default_HandlerB,
    }, // hard fault
    IrqVector {
        handler: Default_HandlerC,
    }, // mem manager
    IrqVector {
        handler: Default_HandlerD,
    }, // bus fault
    IrqVector {
        handler: Default_HandlerE,
    }, // usage
    IrqVector { not_used: 0 },
    IrqVector { not_used: 0 },
    IrqVector { not_used: 0 },
    IrqVector { not_used: 0 },
    IrqVector {
        handler: Default_HandlerF,
    }, // SVC
    IrqVector { not_used: 0 }, // ebug mon
    IrqVector { not_used: 0 },
    IrqVector {
        handler: Default_HandlerG,
    }, // Pend  SV
    IrqVector {
        handler: Default_HandlerH,
    }, // Sys Timer
    IrqVector {
        handler: Default_Handler1,
    }, // WWDG
    IrqVector {
        handler: Default_Handler1,
    }, // PVD
    IrqVector {
        handler: Default_Handler,
    }, // TAMP_STAMP
    IrqVector {
        handler: Default_Handler,
    }, // RTC_WKUP
    IrqVector {
        handler: Default_Handler,
    }, // FLASH
    IrqVector {
        handler: Default_Handler,
    }, // RCC
    IrqVector {
        handler: Default_Handler,
    }, // EXTI0
    IrqVector {
        handler: Default_Handler,
    }, // EXTI1
    IrqVector {
        handler: Default_Handler,
    }, // EXTI2
    IrqVector {
        handler: Default_Handler,
    }, // EXTI3
    IrqVector {
        handler: Default_Handler,
    }, // EXTI4
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream0
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream1
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream2
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream3
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream4
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream5
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream6
    IrqVector {
        handler: Default_Handler,
    }, // ADC
    IrqVector {
        handler: Default_Handler,
    }, // CAN1_TX
    IrqVector {
        handler: Default_Handler,
    }, // CAN1_RX0
    IrqVector {
        handler: Default_Handler,
    }, // CAN1_RX1
    IrqVector {
        handler: Default_Handler,
    }, // CAN1_SCE
    IrqVector {
        handler: Default_Handler,
    }, // EXTI9_5
    IrqVector {
        handler: Default_Handler1,
    }, // TIM1_BRK_TIM9
    IrqVector {
        handler: TIM1_UP_TIM10_IRQHandler,
    }, // TIM1_UP_TIM10
    IrqVector {
        handler: Default_Handler2,
    }, // TIM1_TRG_COM_TIM11
    IrqVector {
        handler: Default_Handler3,
    }, // TIM1_CC
    IrqVector {
        handler: TIM2_IRQHandler,
    }, // TIM2
    IrqVector {
        handler: Default_Handler,
    }, // TIM3
    IrqVector {
        handler: Default_Handler,
    }, // TIM4
    IrqVector {
        handler: Default_Handler,
    }, // I2C1_EV
    IrqVector {
        handler: Default_Handler,
    }, // I2C1_ER
    IrqVector {
        handler: Default_Handler,
    }, // I2C2_EV
    IrqVector {
        handler: Default_Handler,
    }, // I2C2_ER
    IrqVector {
        handler: Default_Handler,
    }, // SPI1
    IrqVector {
        handler: Default_Handler,
    }, // SPI2
    IrqVector {
        handler: Default_Handler,
    }, // USART1
    IrqVector {
        handler: Default_Handler,
    }, // USART2
    IrqVector {
        handler: Default_Handler,
    }, // USART3
    IrqVector {
        handler: Default_Handler,
    }, // EXTI15_10
    IrqVector {
        handler: Default_Handler,
    }, // RTC_Alarm
    IrqVector {
        handler: Default_Handler,
    }, // OTG_FS_WKUP
    IrqVector {
        handler: Default_Handler,
    }, // TIM8_BRK_TIM12
    IrqVector {
        handler: Default_Handler,
    }, // TIM8_UP_TIM13
    IrqVector {
        handler: Default_Handler,
    }, // TIM8_TRG_COM_TIM14
    IrqVector {
        handler: Default_Handler,
    }, // TIM8_CC
    IrqVector {
        handler: Default_Handler,
    }, // DMA1_Stream7
    IrqVector {
        handler: Default_Handler,
    }, // FSMC
    IrqVector {
        handler: Default_Handler,
    }, // SDIO
    IrqVector {
        handler: Default_Handler,
    }, // TIM5
    IrqVector {
        handler: Default_Handler,
    }, // SPI3
    IrqVector {
        handler: Default_Handler,
    }, // UART4
    IrqVector {
        handler: Default_Handler,
    }, // UART5
    IrqVector {
        handler: Default_Handler,
    }, // TIM6_DAC
    IrqVector {
        handler: Default_Handler,
    }, // TIM7
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream0
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream1
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream2
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream3
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream4
    IrqVector {
        handler: Default_Handler,
    }, // ETH
    IrqVector {
        handler: Default_Handler,
    }, // ETH_WKUP
    IrqVector {
        handler: Default_Handler,
    }, // CAN2_TX
    IrqVector {
        handler: Default_Handler,
    }, // CAN2_RX0
    IrqVector {
        handler: Default_Handler,
    }, // CAN2_RX1
    IrqVector {
        handler: Default_Handler,
    }, // CAN2_SCE
    IrqVector {
        handler: Default_Handler,
    }, // OTG_FS
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream5
    IrqVector {
        handler: Default_Handler,
    }, // DMA2_Stream6
    IrqVector {
        handler: Dma_Uart1_Handler,
    }, // DMA2_Stream7
    IrqVector {
        handler: Default_Handler,
    }, // USART6
    IrqVector {
        handler: Default_Handler,
    }, // I2C3_EV
    IrqVector {
        handler: Default_Handler,
    }, // I2C3_ER
    IrqVector {
        handler: Default_Handler,
    }, // OTG_HS_EP1_OUT
    IrqVector {
        handler: Default_Handler,
    }, // OTG_HS_EP1_IN
    IrqVector {
        handler: Default_Handler,
    }, // OTG_HS_WKUP
    IrqVector {
        handler: Default_Handler,
    }, // OTG_HS
    IrqVector {
        handler: Default_Handler,
    }, // DCMI
    IrqVector {
        handler: Default_Handler,
    }, // CRYP
    IrqVector {
        handler: Default_Handler,
    }, // HASH_RNG
    IrqVector {
        handler: Default_Handler,
    }, // FPU
];

#[cfg(test)]
mod tests {

    #[cfg(feature = "std")]
    #[test]
    fn test_default_handler() {
        use super::Default_Handler;
        Default_Handler();
    }
}
