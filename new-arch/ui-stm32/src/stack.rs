use core::arch::asm;

extern "C" {
    static _stack_start: u32;
    static _stack_end: u32;
}

const STACK_PAINT_VALUE: u32 = 0xcccc_cccc;

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

/// This function provides an estimate of stack usage when used with the "paint-stack" feature of
/// the cortex-m-rt crate.  That feature caues memory to be initialized full of STACK_PAINT_VALUE.
/// This function scans through the whole stack area from bottom to top, looking for the first
/// memory location that does not contain STACK_PAINT_VALUE.
///
/// TODO(RLB) Wrap this in a critical section so that it doesn't get interrupted.
#[inline(never)]
pub fn usage() -> usize {
    unsafe {
        let stack_pointer = stack_ptr();
        let stack_start = &_stack_start as *const u32;
        let stack_end = &_stack_end as *const u32;

        let high_water_mark = {
            let mut curr = stack_end;
            while curr.read_volatile() == STACK_PAINT_VALUE && curr < stack_pointer {
                curr = curr.offset(1);
            }
            curr.offset(-1) as usize
        };

        (stack_start as usize) - high_water_mark
    }
}

/// This function "repaints" the stack, starting from after the current stack.  It fills the entire
/// remainder of main memory with STACK_PAINT_VALUE, so that calling usage() thereafter will report
/// incremental memory usage.
///
/// TODO(RLB) Wrap this in a critical section so that it doesn't get interrupted.
#[inline(never)]
pub fn repaint() {
    unsafe {
        let stack_end = (&_stack_end as *const u32) as *mut u32;

        let curr = stack_ptr().offset(-1) as *mut u32;
        while curr > stack_end {
            curr.write_volatile(STACK_PAINT_VALUE);
        }
    }
}

/// This function measures the stack usage of a specific function by repainting memory from the
/// current stack pointer, then running the function, then measuring how the high-water mark has
/// changed.
#[inline(never)]
pub fn measure<F>(f: F) -> usize
where
    F: Fn(),
{
    repaint();
    let usage_before = usage();
    f();
    let usage_after = usage();
    usage_after - usage_before
}
