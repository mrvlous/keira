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

//! Shared memory segment structures and semaphore primitives.

pub mod types {
    pub const SHM_CMD_INFO: u32 = 1;
    pub const SHM_CMD_GET: u32 = 2;
    pub const SHM_CMD_AT: u32 = 3;
    pub const SHM_CMD_DT: u32 = 4;
    pub const SHM_CMD_RM: u32 = 5;
    pub const SEM_CMD_RM: u32 = 6;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct ShmSegment {
        pub id: u32,
        pub key: u32,
        pub size_bytes: usize,
        pub phys_frame: u64,
        pub attach_count: u32,
        pub owner_pid: u32,
        pub in_use: bool,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Semaphore {
        pub id: u32,
        pub key: u32,
        pub value: i32,
        pub waiters: u32,
        pub in_use: bool,
    }
}

pub use self::types::*;

pub use super::manager::*;
