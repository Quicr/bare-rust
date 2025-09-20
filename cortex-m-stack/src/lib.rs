#![no_std]
#![no_main]

// TODO Doc

use core::arch::asm;

/// This function grabs the current stack pointer
#[inline(always)]
fn stack_ptr() -> *const u32 {
    let x: *const u32;
    unsafe {
        asm!(
            "mov {0}, sp" ,
            out(reg) x,
            options(pure, nomem, nostack),
        );
    }
    x
}

extern "C" {
    static _stack_start: u32;
    static _stack_end: u32;
}

const STACK_PAINT_VALUE: u32 = 0xcccccccc;

/// This function reads the paint that has been installed by cortex-m-rt and by any calls to
/// repaint().  It starts at the end of the stack and reads until it finds a location where the
/// referenced memory is not equal to STACK_PAINT_VALUE.  In other words, it reports the highest
/// memory location such that it and all locations below it are equal to STACK_PAINT_VALUE.
#[inline(always)]
unsafe fn high_water_mark() -> *const u32 {
    let stack_pointer = stack_ptr();
    let stack_end = &_stack_end as *const u32;
    let mut curr = stack_end.offset(1);
    while curr.read_volatile() == STACK_PAINT_VALUE && curr < stack_pointer {
        curr = curr.offset(1);
    }
    curr.offset(-1)
}

/// This function reports the maximum stack usage at any point up to the current moment, as the
/// difference between the high water mark and the start of the stack.
#[inline(always)]
pub fn usage() -> usize {
    cortex_m::interrupt::free(|_cs| {
        let stack_start = unsafe { &_stack_start as *const u32 };
        let hwm = unsafe { high_water_mark() };
        (stack_start as usize) - (hwm as usize)
    })
}

/// This function "repaints" the stack, starting from after the current stack.  It fills the entire
/// remainder of main memory with STACK_PAINT_VALUE, so that calling usage() thereafter will report
/// incremental memory usage.  In other words, this function resets the high water mark to the
/// current top of the stack.
#[inline(always)]
pub fn repaint() {
    cortex_m::interrupt::free(|_cs| unsafe {
        let sp = stack_ptr();
        let hwm = high_water_mark();

        let mut curr = sp.offset(-1) as *mut u32;
        while (curr as *const u32) > hwm {
            curr.write_volatile(STACK_PAINT_VALUE);
            curr = curr.offset(-1);
        }
    });
}

/// This function measures the stack usage of a specific function by repainting memory from the
/// current stack pointer, then running the function, then measuring how the high-water mark has
/// changed.
#[inline(never)]
pub fn measure<F>(f: F) -> usize
where
    F: Fn(),
{
    cortex_m::interrupt::free(|_cs| {
        repaint();
        let usage_before = usage();
        f();
        let usage_after = usage();
        usage_after - usage_before
    })
}
