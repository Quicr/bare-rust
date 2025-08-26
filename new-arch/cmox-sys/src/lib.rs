#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Manual definitions for #define macros that bindgen doesn't capture
// (Only the ones with type casts that bindgen can't handle automatically)
pub const CMOX_INIT_SUCCESS: cmox_init_retval_t = 0x00000000;
pub const CMOX_INIT_FAIL: cmox_init_retval_t = 0x00000001;

pub const CMOX_INIT_TARGET_AUTO: cmox_init_target_t = 0x00000000;
pub const CMOX_INIT_TARGET_F0: cmox_init_target_t = 0x46300000;
pub const CMOX_INIT_TARGET_F1: cmox_init_target_t = 0x46310000;
pub const CMOX_INIT_TARGET_F2: cmox_init_target_t = 0x46320000;
pub const CMOX_INIT_TARGET_F3: cmox_init_target_t = 0x46330000;
pub const CMOX_INIT_TARGET_F4: cmox_init_target_t = 0x46340000;
pub const CMOX_INIT_TARGET_F7: cmox_init_target_t = 0x46370000;
pub const CMOX_INIT_TARGET_H5: cmox_init_target_t = 0x48350000;
pub const CMOX_INIT_TARGET_H7: cmox_init_target_t = 0x48370000;
pub const CMOX_INIT_TARGET_H7AB: cmox_init_target_t = 0x48378000;
pub const CMOX_INIT_TARGET_G0: cmox_init_target_t = 0x47300000;
pub const CMOX_INIT_TARGET_G4: cmox_init_target_t = 0x47340000;
pub const CMOX_INIT_TARGET_L0: cmox_init_target_t = 0x4C300000;
pub const CMOX_INIT_TARGET_L1: cmox_init_target_t = 0x4C310000;
pub const CMOX_INIT_TARGET_L4: cmox_init_target_t = 0x4C340000;
pub const CMOX_INIT_TARGET_L5: cmox_init_target_t = 0x4C350000;
pub const CMOX_INIT_TARGET_WB: cmox_init_target_t = 0x57420000;
pub const CMOX_INIT_TARGET_WBA: cmox_init_target_t = 0x57424100;
pub const CMOX_INIT_TARGET_WL: cmox_init_target_t = 0x574C0000;
