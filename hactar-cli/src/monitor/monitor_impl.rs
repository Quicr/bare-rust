use crate::utility::errors::Result;

#[derive(Debug)]
pub struct MonitorArgs {
    pub port: Option<String>,
    pub baud: u32,
}

pub fn monitor(args: MonitorArgs) -> Result<()> {
    // Stub - will be implemented in Milestone 8
    println!("Monitor command called with: {:?}", args);
    println!("Monitor will be implemented in later milestones");
    Ok(())
}
