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

//! io_uring Completion Queue (CQ) entries and circular ring.

pub mod entry {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default)]
    pub struct CompletionQueueEntry {
        pub user_data: u64,
        pub res: i32,
        pub flags: u32,
    }
}

pub mod ring {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default)]
    pub struct IoCqringOffsets {
        pub head: u32,
        pub tail: u32,
        pub ring_mask: u32,
        pub ring_entries: u32,
        pub overflow: u32,
        pub cqes: u32,
        pub flags: u32,
        pub resv1: u32,
        pub user_addr: u64,
        pub pad: [u64; 2],
    }
}

pub use self::entry::*;
pub use self::ring::*;
