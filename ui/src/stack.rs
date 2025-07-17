//! Stack usage tracking and painting.

#[cfg(target_arch = "arm")]
use core::ptr;

#[cfg(target_arch = "arm")]
use core::arch::asm;

extern "C" {
    static _estack: u8;
    static _heap_start: u8;
    static _stack_reserve_start: u8;
    static _stack_reserve_end: u8;
}

#[allow(dead_code)]
const STACK_PAINT: u32 = 0xc5c5c5c5;
#[allow(dead_code)]
pub const STACK_PAINT_BYTE: u8 = 0xc5; // this is used by startup

#[cfg(target_arch = "arm")]
#[inline(never)]
/// Retrieves the current address of the stack pointer.
fn get_stack_pointer() -> u32 {
    let sp: u32;
    unsafe {
        asm!("mov {}, sp", out(reg) sp);
    }
    sp
}

#[cfg(target_arch = "arm")]
#[inline(never)]
/// Calculates the maximum stack usage since the last repaint
/// and optionally repaints the stack.
pub fn usage(repaint: bool) -> (usize, usize, usize) {
    let start: u32 = ptr::addr_of!(_heap_start) as u32;
    let end: u32 = ptr::addr_of!(_estack) as u32;

    let mut lower_bound = start;
    let mut upper_bound = end - 8;

    while upper_bound - lower_bound > 8 {
        let mid = (lower_bound + (upper_bound - lower_bound) / 2) & !0x3;
        let val1: u32;
        let val2: u32;
        unsafe {
            let addr1 = mid as *const u32;
            val1 = ptr::read_volatile(addr1);
            let addr2 = (mid + 4) as *const u32;
            val2 = ptr::read_volatile(addr2);
        }

        if (val1 == STACK_PAINT) && (val2 == STACK_PAINT) {
            lower_bound = mid;
        } else {
            upper_bound = mid;
        }
    }

    // fine tune the upper bound
    upper_bound = (upper_bound & !0x3) + 4;
    if false {
        loop {
            let val: u32;
            unsafe {
                let addr = upper_bound - 4;
                val = ptr::read_volatile((addr) as *const u32)
            }
            if val != 0xc5c5c5c5 {
                break;
            }
            upper_bound -= 4;
        }
    }

    // repaint the stack
    if repaint {
        let stack_pointer = get_stack_pointer();

        let mut ptr = upper_bound;
        while ptr < stack_pointer {
            unsafe {
                ptr::write(ptr as *mut u32, 0xc5c5c5c5);
            }
            ptr += 4;
        }
    }

    let reserve_start: u32 = ptr::addr_of!(_stack_reserve_start) as u32;
    let reserve_end: u32 = ptr::addr_of!(_stack_reserve_end) as u32;
    let reserved: usize = (reserve_end - reserve_start) as usize;

    let usage = (end - upper_bound) as usize;
    let sp = get_stack_pointer() as usize;
    let current = ((end as usize) - sp) as usize;

    if usage > reserved {
        panic!("Stack overflow");
    }

    (usage, current, reserved)
}

#[cfg(not(target_arch = "arm"))]
pub fn usage(_repaint: bool) -> (usize, usize, usize) {
    (0, 0, 0)
}
