use crate::utility::errors::Result;

#[derive(Debug)]
pub struct FlashArgs {
    pub port: Option<String>,
    pub baud: u32,
    pub chip: String,
    pub binary_path: Option<String>,
    pub use_external_flasher: bool,
}

pub fn flash(args: FlashArgs) -> Result<()> {
    // Stub - will be implemented in Milestone 7
    println!("Flash command called with: {:?}", args);
    println!("Flasher will be implemented in later milestones");
    Ok(())
}
