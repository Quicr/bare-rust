//! Utility functions for the CMOX crate

use crate::error::{FromRetval, Result, UtilsError, UtilsResult};
use cmox_sys::cmox_utils_compare;

/// Compares two buffers in a fault-secure way
pub fn constant_time_eq(buf1: &[u8], buf2: &[u8]) -> Result<()> {
    let mut fault = 0xffffffff;

    unsafe {
        UtilsResult::from_rv(cmox_utils_compare(
            buf1.as_ptr(),
            buf1.len() as u32,
            buf2.as_ptr(),
            buf2.len() as u32,
            &mut fault,
        ))?;
    };

    (fault != cmox_sys::CMOX_UTILS_AUTH_SUCCESS)
        .then_some(())
        .ok_or(UtilsError::AuthFail.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
        assert!(!constant_time_eq(b"hell", b"hello"));
    }
}
