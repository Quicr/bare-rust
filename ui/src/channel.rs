//! # Channel Module
//!
//! This module provides a asynchronous channel, with multiple producers and single consumer.
//! It follows the standard Rust `std::sync::mpsc` API.
//!
//! It includes functionality for creating channels, sending messages, and receiving messages.
//!
//! ## Structs
//!
//! - `Sender`: Represents the sending side of a message channel.
//! - `Receiver`: Represents the receiving side of a message channel.
//!
//! ## Functions
//!
//! - `channel`: Creates a new message channel and returns a tuple containing the sender and receiver.
//!
//! ## Usage
//!
//! To use this module, create a channel using the `channel` function,
//! which returns a `Sender` and `Receiver`.
//! You can then use the `send` method on the `Sender` to send messages
//! and the `recv` method on the `Receiver` to receive messages.
//!
//! ## Example
//!
//! ```rust
//! use crate::channel::mpsc;
//! use crate::msg::Msg;
//!
//! let (mut sender, receiver): (mpsc::Sender<Msg>, mpsc::Receiver<Msg>) = mpsc::channel();
//!
//! sender.send(Msg::PttButton(true));
//!
//! loop {
//!     let msg = receiver.recv();
//!     if msg == Msg::None {
//!         break;
//!     }
//!     match msg {
//!         Msg::None => println!("None"),
//!         Msg::PttButton(state) => println!("PttButton: {}", state),
//!         _ => {}
//!     }
//! }
//! ```
use crate::msg::Msg;

pub mod mpsc {
    use super::Msg;
    use crate::vec::VecMsg;

    const Q_SIZE: usize = 10;

    const NUM_QUEUES: usize = 2;

    static mut Q: [VecMsg<Q_SIZE>; NUM_QUEUES] = [{ VecMsg::new() }, { VecMsg::new() }];

    static mut NUM_Q: usize = 0;

    /// A sender for a message channel.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Sender<T> {
        ch: usize,
        _marker: *const T,
    }

    impl<T> Sender<T> {
        /// Sends a message to the channel.
        ///
        /// # Arguments
        ///
        /// * `msg` - The message to send.
        ///
        pub fn send(&self, msg: Msg) {
            let ch = self.ch;
            unsafe { Q[ch].push(msg) };
        }
    }

    /// A receiver for a message channel.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Receiver<T> {
        ch: usize,
        _marker: *const T,
    }

    impl<T> Receiver<T> {
        /// Receives a message from the channel.
        ///
        /// # Returns
        ///
        /// * `Msg` - The received message.
        ///            If the queue is empty, returns `Msg::None`.
        ///
        pub fn recv(&self) -> Msg {
            let ch = self.ch;

            if unsafe { Q[ch].len() } == 0 {
                return Msg::None;
            }
            let msg = unsafe { Q[ch].pop() };

            msg
        }
    }


    /// Creates a new message channel.
    ///
    /// # Returns
    ///
    /// * `(Sender<T>, Receiver<T>)` - A tuple containing the sender and receiver for the channel.
    ///
    pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let ch = unsafe { NUM_Q };
        unsafe { NUM_Q += 1 };
        if ch >= NUM_QUEUES {
            panic!("Too many channels");
        }

        let sender = Sender {
            ch: ch,
            _marker: core::ptr::null(),
            //_marker: core::marker::PhantomData
        } as Sender<T>;
        let receiver = Receiver {
            ch: ch,
            _marker: core::ptr::null(),
            //_marker: core::marker::PhantomData
        } as Receiver<T>;

        (sender, receiver)
    }
}

