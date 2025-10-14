// Uploader trait and implementations
// Will be implemented in Milestones 3-6

use crate::utility::errors::Result;

pub trait Uploader {
    fn flash_select(&mut self) -> Result<()>;
    fn flash_firmware(&mut self, binary_path: &str) -> Result<bool>;
}
