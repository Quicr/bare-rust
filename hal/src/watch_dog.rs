use core::ptr;

use super::cpu;
pub use super::cpu::*;

pub fn init() {}

pub fn is_enabled() -> bool {
    let v = cpu::read!( WWDG.cr[WDGA;1] );
    v != 0
}

#[cfg(feature = "stm32f072")]
pub fn start() {
    cpu::write!( RCC.apb1enr[WWDGEN;1], 1 ); // turn on clock for WWDG

    cpu::write!( DBGMCU.apb1_fz[DBG_WWDG_STOP;1],0b1);

    cpu::write!( WWDG.cfr[EWI;1]  , 0b1 ); // interupt
    cpu::write!( WWDG.cfr[WDGTB;2]  , 0b11 ); // Div 8 prescaler
    cpu::write!( WWDG.cfr[W;7]  , 0x7F ); // window range

    cpu::write!( WWDG.cr[T;7]  , 0x7F ); // set counter
    cpu::write!( WWDG.cr[WDGA;1] , 0b1 ); // enable

    // with 48 MHz pcb clock, 1/8 prescaler,  0x7F counter
    // 48_000_000 / 4096 / 8 / ( 0x7F +1 ) = 11.4 Hz = 87.7 ms
}

#[cfg(feature = "stm32f405")]
pub fn start() {
    cpu::write!(RCC.apb1enr[WWDGEN;1], 1 ); // turn on clock for WWDG

    cpu::write!( DBG.dbgmcu_apb1_fz[DBG_WWDG_STOP;1],0b1);

    cpu::write!( WWDG.cfr[EWI;1]  , 0b0 ); // no interupt
    cpu::write!( WWDG.cfr[WDGTB0;2]  , 0b11 ); // Div 8 prescaler
    cpu::write!( WWDG.cfr[W;7]  , 0x7F ); // set window range

    cpu::write!( WWDG.cr[T;7]  , 0x7F ); // set counter
    cpu::write!( WWDG.cr[WDGA;1] , 0b1 ); // enable
}

/// Resets the WWDG (Window Watchdog) counter to prevent a system reset.
///
pub fn alive() {
    cpu::write!(WWDG.cr[T;7], 0x7F);
}
