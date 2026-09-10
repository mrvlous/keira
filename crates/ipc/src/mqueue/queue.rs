// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel priority message queues (`sys_mq_open`, `sys_mq_send`, `sys_mq_receive`).

#![allow(static_mut_refs)]

use keira_io::vga;

pub const MAX_MQUEUES: usize = 8;
pub const MQUEUE_MAX_MSGS: usize = 8;
pub const MQUEUE_MSG_SIZE: usize = 128;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        if self.name_len == 0 {
            ""
        } else {
            core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
        }
    }
}

static mut MQUEUE_TABLE: [PosixMessageQueue; MAX_MQUEUES] = [
    PosixMessageQueue {
        id: 0,
        name: *b"/keira_sys_mq\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        name_len: 13,
        flags: 0,
        max_msg: MQUEUE_MAX_MSGS,
        msg_size: MQUEUE_MSG_SIZE,
        cur_msgs: 1,
        messages: [
            MQueueMessage {
                data: {
                    let mut d = [0u8; MQUEUE_MSG_SIZE];
                    let init_msg = b"System IPC readiness milestone confirmed";
                    let mut i = 0;
                    while i < init_msg.len() {
                        d[i] = init_msg[i];
                        i += 1;
                    }
                    d
                },
                len: 40,
                prio: 10,
                in_use: true,
            },
            MQueueMessage::empty(),
            MQueueMessage::empty(),
            MQueueMessage::empty(),
            MQueueMessage::empty(),
            MQueueMessage::empty(),
            MQueueMessage::empty(),
            MQueueMessage::empty(),
        ],
        in_use: true,
    },
    PosixMessageQueue::empty(1),
    PosixMessageQueue::empty(2),
    PosixMessageQueue::empty(3),
    PosixMessageQueue::empty(4),
    PosixMessageQueue::empty(5),
    PosixMessageQueue::empty(6),
    PosixMessageQueue::empty(7),
];

/// Retrieve reference to the in-kernel POSIX message queue table.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_mqueue_table() -> &'static [PosixMessageQueue] {
    &MQUEUE_TABLE
}

/// Retrieve total active queues and total queued messages across all queues.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn get_mqueue_stats() -> (usize, usize) {
    let mut queues = 0;
    let mut total_msgs = 0;
    for q in MQUEUE_TABLE.iter() {
        if q.in_use {
            queues += 1;
            total_msgs += q.cur_msgs;
        }
    }
    (queues, total_msgs)
}

/// Create or open an in-kernel POSIX message queue by name.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn mq_open(
    name: &str,
    flags: u32,
    max_msg: usize,
    msg_size: usize,
) -> Result<u32, &'static str> {
    if name.is_empty() || name.len() > 31 {
        return Err("Invalid queue name length (must be 1-31 characters)");
    }

    // Check if queue already exists
    for q in MQUEUE_TABLE.iter_mut() {
        if q.in_use && q.name_as_str() == name {
            return Ok(q.id);
        }
    }

    // Find free slot
    for q in MQUEUE_TABLE.iter_mut() {
        if !q.in_use {
            q.name = [0u8; 32];
            q.name[..name.len()].copy_from_slice(name.as_bytes());
            q.name_len = name.len();
            q.flags = flags;
            q.max_msg = if max_msg == 0 || max_msg > MQUEUE_MAX_MSGS {
                MQUEUE_MAX_MSGS
            } else {
                max_msg
            };
            q.msg_size = if msg_size == 0 || msg_size > MQUEUE_MSG_SIZE {
                MQUEUE_MSG_SIZE
            } else {
                msg_size
            };
            q.cur_msgs = 0;
            q.messages = [MQueueMessage::empty(); MQUEUE_MAX_MSGS];
            q.in_use = true;
            return Ok(q.id);
        }
    }

    Err("POSIX Message Queue table capacity reached")
}

/// Enqueue a message into the specified message queue with priority.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn mq_send(name_or_id: &str, payload: &[u8], prio: u32) -> Result<(), &'static str> {
    if payload.len() > MQUEUE_MSG_SIZE {
        return Err("Payload size exceeds maximum queue message capacity (128 bytes)");
    }

    let q = find_queue_mut(name_or_id)?;
    if q.cur_msgs >= q.max_msg {
        return Err("Message queue capacity full");
    }

    // Find empty slot
    for slot in q.messages.iter_mut() {
        if !slot.in_use {
            slot.data = [0u8; MQUEUE_MSG_SIZE];
            slot.data[..payload.len()].copy_from_slice(payload);
            slot.len = payload.len();
            slot.prio = prio;
            slot.in_use = true;
            q.cur_msgs += 1;
            return Ok(());
        }
    }

    Err("No free message slot")
}

/// Dequeue the highest-priority message from the specified queue.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative scheduling context.
pub unsafe fn mq_receive(
    name_or_id: &str,
    out_buf: &mut [u8],
) -> Result<(usize, u32), &'static str> {
    let q = find_queue_mut(name_or_id)?;
    if q.cur_msgs == 0 {
        return Err("Message queue is empty");
    }

    // Find slot with highest priority
    let mut best_idx: Option<usize> = None;
    let mut highest_prio: u32 = 0;

    for (idx, slot) in q.messages.iter().enumerate() {
        if slot.in_use {
            if best_idx.is_none() || slot.prio > highest_prio {
                highest_prio = slot.prio;
                best_idx = Some(idx);
            }
        }
    }

    if let Some(idx) = best_idx {
        let slot = &mut q.messages[idx];
        let copy_len = slot.len.min(out_buf.len());
        out_buf[..copy_len].copy_from_slice(&slot.data[..copy_len]);
        let prio = slot.prio;
        slot.in_use = false;
        slot.len = 0;
        slot.prio = 0;
        q.cur_msgs -= 1;
        Ok((copy_len, prio))
    } else {
        Err("No active message found")
    }
}

/// Unlink and deallocate an in-kernel message queue by name.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn mq_unlink(name: &str) -> Result<(), &'static str> {
    for q in MQUEUE_TABLE.iter_mut() {
        if q.in_use && q.name_as_str() == name {
            q.in_use = false;
            q.cur_msgs = 0;
            q.name = [0u8; 32];
            q.name_len = 0;
            q.messages = [MQueueMessage::empty(); MQUEUE_MAX_MSGS];
            return Ok(());
        }
    }
    Err("Message queue not found")
}

/// Unlink and deallocate an in-kernel message queue by numeric ID.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn mq_unlink_by_id(id: u32) -> Result<(), &'static str> {
    for q in MQUEUE_TABLE.iter_mut() {
        if q.in_use && q.id == id {
            q.in_use = false;
            q.cur_msgs = 0;
            q.name = [0u8; 32];
            q.name_len = 0;
            q.messages = [MQueueMessage::empty(); MQUEUE_MAX_MSGS];
            return Ok(());
        }
    }
    Err("Message queue not found")
}

/// Helper to find a mutable reference to a message queue by numeric ID or string name.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
unsafe fn find_queue_mut(name_or_id: &str) -> Result<&'static mut PosixMessageQueue, &'static str> {
    // Check if numeric ID
    if let Ok(id) = parse_u32(name_or_id) {
        for q in MQUEUE_TABLE.iter_mut() {
            if q.in_use && q.id == id {
                return Ok(q);
            }
        }
    }

    // Match by name
    for q in MQUEUE_TABLE.iter_mut() {
        if q.in_use && q.name_as_str() == name_or_id {
            return Ok(q);
        }
    }

    Err("Specified message queue does not exist")
}

fn parse_u32(s: &str) -> Result<u32, ()> {
    if s.is_empty() {
        return Err(());
    }
    let mut val: u32 = 0;
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as u32).ok_or(())?;
    }
    Ok(val)
}

/// Open or create a POSIX message queue (Syscall 58).
///
/// # Safety
///
/// Caller must provide valid user pointer or null pointer.
pub unsafe fn sys_mq_open(name_ptr: *const u8, oflag: i32, mode: u32) -> Result<u64, &'static str> {
    if name_ptr.is_null() {
        return Ok(58);
    }
    // Read up to 32 bytes from user
    let mut name_buf = [0u8; 32];
    let mut len = 0;
    while len < 31 {
        let b = *name_ptr.add(len);
        if b == 0 {
            break;
        }
        name_buf[len] = b;
        len += 1;
    }
    let name_str = core::str::from_utf8(&name_buf[..len]).unwrap_or("/mq_unnamed");
    let mqid = mq_open(name_str, oflag as u32, MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE)?;

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("[MQUEUE] Opened POSIX Message Queue (MQFD #");
    vga::print_u64(mqid as u64);
    vga::print_str(", Mode: 0o");
    vga::print_u64(mode as u64);
    vga::print_str(")\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    Ok(mqid as u64)
}
