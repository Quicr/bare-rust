//! Metrics module that holds all the metrics for the system.

use crate::tasks::MAX_TASKS;

/// Structure to hold all the metrics for the system.
pub struct Metrics {
    pub task_name: [[u8; 8]; MAX_TASKS],
    pub task_run_count: [u32; MAX_TASKS],
    pub task_max_stack: [u32; MAX_TASKS],
    pub task_max_duration_us: [u32; MAX_TASKS],
}

impl Metrics {
    /// Creates a new `Metrics` instance with all values initialized to zero.
    pub fn new() -> Self {
        Metrics {
            task_name: [[0; 8]; MAX_TASKS],
            task_run_count: [0; MAX_TASKS],
            task_max_stack: [0; MAX_TASKS],
            task_max_duration_us: [0; MAX_TASKS],
        }
    }
}
