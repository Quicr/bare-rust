//! The `no_task` module contains the implementation of a placeholder task.
//! This task is used when no task is assigned and should never be run.

use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;

/// Structure representing the NoTask.
pub struct NoTask {}

/// Information about the NoTask.
const NO_TASK_INFO: TaskInfo = TaskInfo {
    name: b"NoTask__",
    run_every_us: 0,
    time_budget_us: 0,
    mem_budget_bytes: 0,
};

impl Task for NoTask {
    /// Method to execute the NoTask.
    /// This method should never be called and will panic if invoked.
    fn run(
        &self,
        _sender: &mut crate::mpsc::Sender<Msg>,
        _bsp: &mut bsp::BSP,
        _task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
        panic!("NoTask should never run");
    }

    /// Returns the information about the NoTask.
    #[allow(dead_code)]
    fn info(&self) -> &'static TaskInfo {
        &NO_TASK_INFO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_no_task() {
        let no_task = tasks::no_task::NoTask {};
        let info = no_task.info();
        assert_eq!(info.mem_budget_bytes, 0);
    }
}
