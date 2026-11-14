// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! io_uring Submission Queue (SQ) entries and circular ring.

pub mod entry {
    pub const IOSQE_FIXED_FILE: u8 = 1 << 0;
    pub const IOSQE_IO_DRAIN: u8 = 1 << 1;
    pub const IOSQE_IO_LINK: u8 = 1 << 2;
    pub const IOSQE_IO_HARDLINK: u8 = 1 << 3;
    pub const IOSQE_ASYNC: u8 = 1 << 4;
    pub const IOSQE_BUFFER_SELECT: u8 = 1 << 5;

    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct SubmissionQueueEntry {
        pub opcode: u8,
        pub flags: u8,
        pub ioprio: u16,
        pub fd: i32,
        pub off: u64,
        pub addr: u64,
        pub len: u32,
        pub rw_flags: u32,
        pub user_data: u64,
        pub buf_index: u16,
        pub personality: u16,
        pub splice_fd_in: i32,
        pub pad2: [u64; 2],
    }

    impl Default for SubmissionQueueEntry {
        fn default() -> Self {
            Self {
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
            }
        }
    }
}

pub mod ring {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default)]
    pub struct IoSqringOffsets {
        pub head: u32,
        pub tail: u32,
        pub ring_mask: u32,
        pub ring_entries: u32,
        pub flags: u32,
        pub dropped: u32,
        pub array: u32,
        pub resv1: u32,
        pub user_addr: u64,
        pub pad: [u64; 2],
    }
}

pub use self::entry::*;
pub use self::ring::*;
