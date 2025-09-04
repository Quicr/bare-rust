//! This module handles the keyboard task for the application.
//!
//! The keyboard task is responsible for reading the state of the keyboard and sending messages
//! when the state changes.
//!
//!
//! The task periodically hecks the state of the rows to detect key presses and releases.
//! When a key press or release is detected, a message is sent to notify other parts of the system.
//!

use super::{Task, TaskData};
use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::tasks::TaskInfo;

/// Structure representing the keyboard task.
pub struct KeyboardTask {}

/// Information about the keyboard task.
const KEYBOARD_TASK_INFO: TaskInfo = TaskInfo {
    name: b"Keyboard",
    run_every_us: 10_000,
    time_budget_us: 10_000,
    mem_budget_bytes: 500,
};

impl Task for KeyboardTask {
    /// Method to execute the keyboard task.
    /// Reads the state of the keyboard and sends a message if the state has changed.
    fn run(
        &self,
        sender: &mut crate::mpsc::Sender<Msg>,
        bsp: &mut bsp::BSP,
        _task_data: &mut TaskData,
        _metrics: &mut Metrics,
    ) {
        // this uses the PTT button to mock the keyboard
        let button_moch_keyboard = true;
        if button_moch_keyboard {
            let (state, changed) = bsp.buttons.read_ptt();
            if changed {
                if state {
                    let keyboard_msg = Msg::Keyboard { key: 'A' };
                    sender.send(keyboard_msg);
                } else {
                    let keyboard_msg = Msg::Keyboard { key: '\r' };
                    sender.send(keyboard_msg);
                }
            }
        }

        let key = bsp.keyboard.get_key();
        if key != 0 {
            let keyboard_msg = Msg::Keyboard { key: key as char };
            sender.send(keyboard_msg);
        }

        // check if key from serial port
        if !hal::uart::empty1() {
            let c: u8;
            c = hal::uart::read1();
            let echo: bool = true; // TODO - turn off echo
            if echo && (c != 0) {
                hal::uart::write1(c);
            }

            let keyboard_msg = Msg::Keyboard { key: c as char };
            sender.send(keyboard_msg);
        }
    }

    /// Returns the information about the keyboard task.
    fn info(&self) -> &'static TaskInfo {
        &KEYBOARD_TASK_INFO
    }
}
