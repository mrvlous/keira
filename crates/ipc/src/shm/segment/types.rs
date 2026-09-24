// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Shared memory segment and semaphore descriptors and commands.

pub const SHM_CMD_INFO: u32 = 1;
pub const SHM_CMD_GET: u32 = 2;
pub const SHM_CMD_AT: u32 = 3;
pub const SHM_CMD_DT: u32 = 4;
pub const SHM_CMD_RM: u32 = 5;
pub const SEM_CMD_RM: u32 = 6;

/// Descriptor representing an allocated POSIX shared memory page frame segment.
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

/// Descriptor representing an in-kernel counting semaphore.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Semaphore {
    pub id: u32,
    pub key: u32,
    pub value: i32,
    pub waiters: u32,
    pub in_use: bool,
}
