// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! io_uring execution runtime and asynchronous dispatch engine.

pub mod runtime;
pub mod types;

pub use runtime::{
    enter_ring, enter_ring_ext, get_ring, get_ring_mut, process_single_sqe, setup_ring,
    setup_ring_ext, CQ_ENTRIES, RINGS, SQ_ENTRIES,
};
pub use types::{
    IoUringInstance, IoUringParams, IoUringRing, IORING_ENTER_GETEVENTS, IORING_ENTER_SQ_WAIT,
    IORING_ENTER_SQ_WAKEUP, IORING_FEAT_NODROP, IORING_FEAT_SINGLE_MMAP, IORING_FEAT_SUBMIT_STABLE,
    IORING_OP_CLOSE, IORING_OP_FSYNC, IORING_OP_NOP, IORING_OP_POLL_ADD, IORING_OP_POLL_REMOVE,
    IORING_OP_READ, IORING_OP_READV, IORING_OP_READ_FIXED, IORING_OP_RECVMSG, IORING_OP_SENDMSG,
    IORING_OP_STATX, IORING_OP_SYNC_FILE_RANGE, IORING_OP_TIMEOUT, IORING_OP_WRITE,
    IORING_OP_WRITEV, IORING_OP_WRITE_FIXED, IORING_SETUP_ATTACH_WQ, IORING_SETUP_CLAMP,
    IORING_SETUP_CQSIZE, IORING_SETUP_IOPOLL, IORING_SETUP_SQPOLL, IORING_SETUP_SQ_AFF,
    MAX_IO_URING_INSTANCES, MAX_RING_ENTRIES,
};
