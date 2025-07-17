//! The `buttons_task` module contains the implementation of the button task.
//!
//! This task is responsible for reading the state of a button and sending a message
//! if the button state changes.

use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;

/// Structure representing the button task.
pub struct ButtonTask {}

/// Information about the button task.
const BUTTON_TASK_INFO: TaskInfo = TaskInfo {
    name: b"Button__",
    run_every_us: 10_000,
    time_budget_us: 10_000,
    mem_budget_bytes: 500,
};

impl Task for ButtonTask {
    /// Method to execute the button task.
    /// Reads the state of the button and sends a message if the state has changed.
    fn run(
        &self,
        sender: &mut crate::mpsc::Sender<Msg>,
        bsp: &mut bsp::BSP,
        _task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
        // junk sender.send(crate::msg::Msg::None );
        let (state, changed) = bsp.buttons.read_ptt();
        if changed {
            sender.send(Msg::PttButton(state));
        }
    }

    /// Returns the information about the button task.
    fn info(&self) -> &'static TaskInfo {
        &BUTTON_TASK_INFO
    }
}
