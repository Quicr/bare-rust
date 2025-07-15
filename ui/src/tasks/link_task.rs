use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;
use crate::vec::VecByte;

/// Structure representing the netLink task.
pub struct LinkTask {}

pub struct Data {
    _junk: u32,
}

impl Data {
    /// Creates a new `Data` instance with an empty buffer.
    pub const fn new() -> Self {
        Data { _junk: 0 }
    }
}

/// Information about the netLink task.
const LINK_TASK_INFO: TaskInfo = TaskInfo {
    name: b"Link____",
    run_every_us: 100_000,
    time_budget_us: 10_000,
    mem_budget_bytes: 500,
};

pub fn recv(
    msg: &Msg,
    sender: &mut crate::mpsc::Sender<Msg>,
    _bsp: &mut bsp::BSP,
    task_data: &mut TaskData,
    _metrics: &mut Metrics,
) {
    let _data = &mut task_data.net_link;

    match msg {
        Msg::EncTxtMsgOut {
            object_id,
            group_id,
            track_alias,
            key_id,
            enc_text,
            auth_tag,
        } => {
            let _ = enc_text;

            // Just echo the message back

            let mut txt = VecByte::<160>::new();
            txt.push(b'H');
            txt.push(b'i');

            let msg = Msg::EncTxtMsgIn {
                object_id: *object_id,
                group_id: *group_id,
                track_alias: *track_alias,
                key_id: *key_id,
                enc_text: txt,
                auth_tag: *auth_tag,
            };
            sender.send(msg);
        }
        _ => {}
    }
}

impl Task for LinkTask {
    /// Method to execute the netLink task.
    /// Reads the state of the netLink and sends a message if the state has changed.
    fn run(
        &self,
        _sender: &mut crate::mpsc::Sender<Msg>,
        _bsp: &mut bsp::BSP,
        _task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
    }

    /// Returns the information about the netLink task.
    fn info(&self) -> &'static TaskInfo {
        &LINK_TASK_INFO
    }
}
