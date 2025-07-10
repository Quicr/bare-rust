//! This module contains the definition of the `Msg` enumeration, which represent
//! different types of messages that can be sent between tasks.

#[cfg(feature = "std")]
extern crate std;

use crate::vec::VecByte;

/// Enumeration representing different types of messages. Multiple task can
/// send messages to the dispatcher, which will then dispatch them to other task.
/// Each type of message is represented by a variant of this enumeration.
/// For each type of message, only one task can consume it. However, multiple
/// tasks can send the same type of message.
///
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Msg {
    None,
    /// Represents a message indicating the state of the PTT (Push-To-Talk) button.
    /// The boolean value indicates whether the button is pressed (`true`) or released (`false`).
    PttButton(bool),
    Keyboard {
        key: char,
    },
    TextInput {
        input: VecByte<160>,
    },
    TxtMsgOut {
        object_id: u32,
        group_id: u32,
        track_alias: u128,
        text: VecByte<160>,
    },
    TxtMsgIn {
        object_id: u32,
        group_id: u32,
        track_alias: u128,
        text: VecByte<160>,
    },
    EncTxtMsgOut {
        object_id: u32,
        group_id: u32,
        track_alias: u128,
        key_id: u32,
        enc_text: VecByte<160>,
        auth_tag: u128,
    },
    EncTxtMsgIn {
        object_id: u32,
        group_id: u32,
        track_alias: u128,
        key_id: u32,
        enc_text: VecByte<160>,
        auth_tag: u128,
    },
    PrintClearMsg,
    PrintClearInputMsg,
    PrintMsg {
        text: VecByte<160>,
    },
    PrintInputMsg {
        text: VecByte<160>,
    },
    //Shutdown,
    //AiButton(bool),
    //MoqObject { name: u128, group: u64, id: u64 },
}

impl Default for Msg {
    fn default() -> Self {
        Msg::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msg() {
        // make sure this file shows up in test coverage
        let _msg = Msg::PttButton(true);
    }
}
