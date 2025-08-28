//! Utility functions for the CMOX crate

use crate::{CmoxError, Result};

/// Ensure CMOX library is initialized before calling cryptographic functions
pub(crate) fn ensure_initialized() -> Result<()> {
    if crate::is_initialized() {
        Ok(())
    } else {
        Err(CmoxError::NotInitialized)
    }
}

/// Constant-time comparison using CMOX utilities
/// 
/// This function provides constant-time comparison of two byte slices
/// to prevent timing attacks when comparing cryptographic values.
/// 
/// # Arguments
/// 
/// * `a` - First byte slice
/// * `b` - Second byte slice
/// 
/// # Returns
/// 
/// `true` if the slices are equal, `false` otherwise
/// 
/// # Note
/// 
/// This function will return `false` if the slices have different lengths.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
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