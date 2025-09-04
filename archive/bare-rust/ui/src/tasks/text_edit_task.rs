//! Task to key pess events and edit input text

use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;
use crate::vec::VecByte;

/// Structure representing the textEdit task.
pub struct TextEditTask {}

pub struct Data {
    buffer: VecByte<160>,
}

impl Data {
    /// Creates a new `Data` instance with an empty buffer.
    pub const fn new() -> Self {
        Data {
            buffer: VecByte::<160>::new(),
        }
    }
}

/// Information about the textEdit task.
const TEXTEDIT_TASK_INFO: TaskInfo = TaskInfo {
    name: b"TextEdit",
    run_every_us: 100_000,
    time_budget_us: 10_000,
    mem_budget_bytes: 300,
};

pub fn recv(
    msg: &Msg,
    sender: &mut crate::mpsc::Sender<Msg>,
    _bsp: &mut bsp::BSP,
    task_data: &mut TaskData,
    _metrics: &mut Metrics,
) {
    let data = &mut task_data.text_edit;

    match msg {
        Msg::Keyboard { key } => {
            // Handle the keyboard message here
            if key == &'\r' {
                // Send the input message

                let text_msg = Msg::TextInput {
                    input: data.buffer.clone(),
                };
                sender.send(text_msg);

                // Clear the buffer
                data.buffer.clear();

                let text_msg = Msg::PrintClearInputMsg {};
                sender.send(text_msg);
            } else {
                if data.buffer.len() < data.buffer.capacity() {
                    let k = *key as u8;
                    data.buffer.push(k);

                    let text_msg = Msg::PrintInputMsg {
                        text: data.buffer.clone(),
                    };
                    sender.send(text_msg);
                }
            }
        }
        _ => {
            // Handle other messages if necessary
        }
    }
}

impl Task for TextEditTask {
    /// Method to execute the textEdit task.
    /// Reads the state of the textEdit and sends a message if the state has changed.
    fn run(
        &self,
        _sender: &mut crate::mpsc::Sender<Msg>,
        _bsp: &mut bsp::BSP,
        task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
        let _data = &mut task_data.text_edit;
    }

    /// Returns the information about the textEdit task.
    fn info(&self) -> &'static TaskInfo {
        &TEXTEDIT_TASK_INFO
    }
}
