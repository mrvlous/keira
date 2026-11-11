// SPDX-License-Identifier: GPL-2.0-only

//! Message queue descriptors and priority message slots.

pub mod message;

pub use super::manager::*;
pub use message::{
    MQueueMessage, PosixMessageQueue, MAX_MQUEUES, MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE,
};
