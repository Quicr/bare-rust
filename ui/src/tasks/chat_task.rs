use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;

/// Structure representing the chat task.
pub struct ChatTask {}

pub struct Data {
    group_id: u32,
    object_id: u32,
    track_alias: u128,
}

impl Data {
    /// Creates a new `Data` instance with an empty buffer.
    pub const fn new() -> Self {
        Data {
            track_alias: 123,
            group_id: 45,
            object_id: 67,
        }
    }
}

/// Information about the chat task.
const CHAT_TASK_INFO: TaskInfo = TaskInfo {
    name: b"Chat____",
    run_every_us: 100_000,
    time_budget_us: 10_000,
    mem_budget_bytes: 500,
};

impl Task for ChatTask {
    /// Method to execute the chat task.
    /// Reads the state of the chat and sends a message if the state has changed.
    fn run(
        &self,
        _sender: &mut crate::mpsc::Sender<Msg>,
        _bsp: &mut bsp::BSP,
        _task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
    }

    /// Returns the information about the chat task.
    fn info(&self) -> &'static TaskInfo {
        &CHAT_TASK_INFO
    }
}

pub fn recv(
    msg: &Msg,
    sender: &mut crate::mpsc::Sender<Msg>,
    _bsp: &mut bsp::BSP,
    task_data: &mut TaskData,
    _metrics: &mut Metrics,
) {
    let data = &mut task_data.chat;

    match msg {
        Msg::TextInput { input } => {
            // Handle the text input message here
            let msg = Msg::TxtMsgOut {
                object_id: data.object_id,
                group_id: data.group_id,
                track_alias: data.track_alias,
                text: input.clone(),
            };
            data.object_id += 1;
            sender.send(msg);

            let print_msg = Msg::PrintMsg {
                text: input.clone(),
            };
            sender.send(print_msg);
        }
        Msg::TxtMsgIn {
            object_id,
            group_id,
            track_alias,
            text,
        } => {
            let _ = (object_id, group_id, track_alias);

            let msg = Msg::PrintMsg { text: text.clone() };

            sender.send(msg);
        }
        _ => {}
    }
}
