// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Standard Linux io_uring opcodes, parameters, and ring container structure.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::uring::cq::entry::CompletionQueueEntry;
use crate::uring::cq::ring::IoCqringOffsets;
use crate::uring::sq::entry::SubmissionQueueEntry;
use crate::uring::sq::ring::IoSqringOffsets;

// Standard Linux io_uring Opcodes.
pub const IORING_OP_NOP: u8 = 0;
pub const IORING_OP_READV: u8 = 1;
pub const IORING_OP_WRITEV: u8 = 2;
pub const IORING_OP_FSYNC: u8 = 3;
pub const IORING_OP_READ_FIXED: u8 = 4;
pub const IORING_OP_WRITE_FIXED: u8 = 5;
pub const IORING_OP_POLL_ADD: u8 = 6;
pub const IORING_OP_POLL_REMOVE: u8 = 7;
pub const IORING_OP_SYNC_FILE_RANGE: u8 = 8;
pub const IORING_OP_SENDMSG: u8 = 9;
pub const IORING_OP_RECVMSG: u8 = 10;
pub const IORING_OP_TIMEOUT: u8 = 11;
pub const IORING_OP_CLOSE: u8 = 19;
pub const IORING_OP_STATX: u8 = 21;
pub const IORING_OP_READ: u8 = 22;
pub const IORING_OP_WRITE: u8 = 23;

// Setup flags for io_uring_setup.
pub const IORING_SETUP_IOPOLL: u32 = 1 << 0;
pub const IORING_SETUP_SQPOLL: u32 = 1 << 1;
pub const IORING_SETUP_SQ_AFF: u32 = 1 << 2;
pub const IORING_SETUP_CQSIZE: u32 = 1 << 3;
pub const IORING_SETUP_CLAMP: u32 = 1 << 4;
pub const IORING_SETUP_ATTACH_WQ: u32 = 1 << 5;

// Enter flags for io_uring_enter.
pub const IORING_ENTER_GETEVENTS: u32 = 1 << 0;
pub const IORING_ENTER_SQ_WAKEUP: u32 = 1 << 1;
pub const IORING_ENTER_SQ_WAIT: u32 = 1 << 2;

// Kernel-supported io_uring feature flags.
pub const IORING_FEAT_SINGLE_MMAP: u32 = 1 << 0;
pub const IORING_FEAT_NODROP: u32 = 1 << 1;
pub const IORING_FEAT_SUBMIT_STABLE: u32 = 1 << 2;

// System ring limits.
pub const MAX_IO_URING_INSTANCES: usize = 8;
pub const MAX_RING_ENTRIES: usize = 64;

/// Parameters passed to and returned from `io_uring_setup`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct IoUringParams {
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub flags: u32,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    pub features: u32,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_off: IoSqringOffsets,
    pub cq_off: IoCqringOffsets,
}

/// A single live instance of an `io_uring` ring buffer channel.
pub struct IoUringRing {
    pub id: u32,
    pub in_use: bool,
    pub flags: u32,
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub sq_mask: u32,
    pub cq_mask: u32,

    pub sq_head: AtomicU32,
    pub sq_tail: AtomicU32,
    pub cq_head: AtomicU32,
    pub cq_tail: AtomicU32,

    pub sqes: [SubmissionQueueEntry; MAX_RING_ENTRIES],
    pub cqes: [CompletionQueueEntry; MAX_RING_ENTRIES],

    pub submitted_count: AtomicU32,
    pub completed_count: AtomicU32,
    pub overflow_count: AtomicU32,
}

pub type IoUringInstance = IoUringRing;

impl IoUringRing {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            in_use: false,
            flags: 0,
            sq_entries: 0,
            cq_entries: 0,
            sq_mask: 0,
            cq_mask: 0,
            sq_head: AtomicU32::new(0),
            sq_tail: AtomicU32::new(0),
            cq_head: AtomicU32::new(0),
            cq_tail: AtomicU32::new(0),
            sqes: [SubmissionQueueEntry {
                opcode: 0,
                flags: 0,
                ioprio: 0,
                fd: -1,
                off: 0,
                addr: 0,
                len: 0,
                rw_flags: 0,
                user_data: 0,
                buf_index: 0,
                personality: 0,
                splice_fd_in: -1,
                pad2: [0; 2],
            }; MAX_RING_ENTRIES],
            cqes: [CompletionQueueEntry {
                user_data: 0,
                res: 0,
                flags: 0,
            }; MAX_RING_ENTRIES],
            submitted_count: AtomicU32::new(0),
            completed_count: AtomicU32::new(0),
            overflow_count: AtomicU32::new(0),
        }
    }

    /// Reset ring state for new setup.
    pub fn reset(&mut self, entries: u32, flags: u32) {
        let actual_sq = entries
            .clamp(1, MAX_RING_ENTRIES as u32)
            .next_power_of_two();
        let actual_cq = (actual_sq * 2).clamp(2, MAX_RING_ENTRIES as u32);

        self.in_use = true;
        self.flags = flags;
        self.sq_entries = actual_sq;
        self.cq_entries = actual_cq;
        self.sq_mask = actual_sq - 1;
        self.cq_mask = actual_cq - 1;

        self.sq_head.store(0, Ordering::Relaxed);
        self.sq_tail.store(0, Ordering::Relaxed);
        self.cq_head.store(0, Ordering::Relaxed);
        self.cq_tail.store(0, Ordering::Relaxed);

        self.submitted_count.store(0, Ordering::Relaxed);
        self.completed_count.store(0, Ordering::Relaxed);
        self.overflow_count.store(0, Ordering::Relaxed);

        for sqe in self.sqes.iter_mut() {
            *sqe = SubmissionQueueEntry::default();
        }
        for cqe in self.cqes.iter_mut() {
            *cqe = CompletionQueueEntry::default();
        }
    }
}
