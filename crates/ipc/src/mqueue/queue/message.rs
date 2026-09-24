// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX message queue structures and priority message buffer descriptors.

pub const MAX_MQUEUES: usize = 8;
pub const MQUEUE_MAX_MSGS: usize = 16;
pub const MQUEUE_MSG_SIZE: usize = 128;

/// Single message payload and priority in an active queue.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MQueueMessage {
    pub data: [u8; MQUEUE_MSG_SIZE],
    pub len: usize,
    pub prio: u32,
    pub in_use: bool,
}

impl MQueueMessage {
    pub const fn empty() -> Self {
        Self {
            data: [0u8; MQUEUE_MSG_SIZE],
            len: 0,
            prio: 0,
            in_use: false,
        }
    }
}

/// In-kernel POSIX priority message queue descriptor.
#[derive(Copy, Clone, Debug)]
pub struct PosixMessageQueue {
    pub id: u32,
    pub name: [u8; 32],
    pub name_len: usize,
    pub flags: u32,
    pub max_msg: usize,
    pub msg_size: usize,
    pub cur_msgs: usize,
    pub messages: [MQueueMessage; MQUEUE_MAX_MSGS],
    pub in_use: bool,
}

impl PosixMessageQueue {
    pub const fn empty(id: u32) -> Self {
        Self {
            id,
            name: [0u8; 32],
            name_len: 0,
            flags: 0,
            max_msg: MQUEUE_MAX_MSGS,
            msg_size: MQUEUE_MSG_SIZE,
            cur_msgs: 0,
            messages: [MQueueMessage::empty(); MQUEUE_MAX_MSGS],
            in_use: false,
        }
    }

    pub fn name_as_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}
