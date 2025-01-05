//! # Netlink Module
//!
//! This module provides functionality for sending and receiving messages to the NET CPU.

extern crate hal;

#[cfg(feature = "std")]
extern crate std;

/// Enum representing different types of Netlink messages.
pub enum NetlinkMessage {
    None,
    OutMoqObject {
        object_id: u32,
        group_id: u32,
        key_id: u32,
        track_alias: u128,

        enc_data_len: u32,
        enc_data: [u32; 64],
    },
    InMoqObject {
        object_id: u32,
        group_id: u32,
        key_id: u32,
        track_alias: u128,
        
        enc_data_len: u32,
        enc_data: [u32; 64],
    },
    FetchMoqObject {
        object_id: u32,
        group_id: u32,
        track_id: u128,
    },
    SubMoqObject {
        track_alias: u128,
    },
}

pub struct Netlink {}

impl crate::netlink::Netlink {
    #[inline(never)]
    pub fn new() -> Self {
        crate::netlink::Netlink {}
    }

    #[inline(never)]
    pub fn init(&self) {}

    /// Queue a message to be sent to NET CPU
    pub fn send(&self, _message: NetlinkMessage) {
        // Send a message
    }

    // Check if there is a message from NET CPU and if so return it.
    // If there is no message, return None.
    pub fn receive(&self) -> NetlinkMessage {
        NetlinkMessage::None
    }
}
