// Hactar device scanning functionality
// Will be fully implemented in Milestone 2

use crate::utility::errors::{HactarError, Result};

pub fn scan_for_hactars() -> Result<Vec<String>> {
    // Stub - will be implemented in Milestone 2
    Ok(vec![])
}

pub fn select_hactar_port() -> Result<String> {
    // Stub - will be implemented in Milestone 2
    Err(HactarError::NoDevicesFound)
}
