// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! In-kernel shared memory segments and counting semaphores registry table.

#![allow(static_mut_refs)]

use crate::shm::segment::types::{Semaphore, ShmSegment};

pub static mut SHM_TABLE: [ShmSegment; 4] = [
    ShmSegment {
        id: 0,
        key: 0x12344321,
        size_bytes: 4096,
        phys_frame: 0x70000000,
        attach_count: 1,
        owner_pid: 1,
        in_use: true,
    },
    ShmSegment {
        id: 1,
        key: 0x56788765,
        size_bytes: 8192,
        phys_frame: 0x70001000,
        attach_count: 2,
        owner_pid: 2,
        in_use: true,
    },
    ShmSegment {
        id: 2,
        key: 0,
        size_bytes: 0,
        phys_frame: 0,
        attach_count: 0,
        owner_pid: 0,
        in_use: false,
    },
    ShmSegment {
        id: 3,
        key: 0,
        size_bytes: 0,
        phys_frame: 0,
        attach_count: 0,
        owner_pid: 0,
        in_use: false,
    },
];

pub static mut SEM_TABLE: [Semaphore; 4] = [
    Semaphore {
        id: 0,
        key: 0x10002000,
        value: 1,
        waiters: 0,
        in_use: true,
    },
    Semaphore {
        id: 1,
        key: 0x30004000,
        value: 5,
        waiters: 0,
        in_use: true,
    },
    Semaphore {
        id: 2,
        key: 0,
        value: 0,
        waiters: 0,
        in_use: false,
    },
    Semaphore {
        id: 3,
        key: 0,
        value: 0,
        waiters: 0,
        in_use: false,
    },
];

/// Retrieve active Shared Memory segments table.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn get_shm_table() -> &'static [ShmSegment] {
    &SHM_TABLE
}

/// Retrieve active Counting Semaphores table.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn get_sem_table() -> &'static [Semaphore] {
    &SEM_TABLE
}

/// Create or locate a shared memory segment of given size (Syscall 28: shmget).
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn create_shm(size: usize) -> Result<usize, &'static str> {
    for seg in SHM_TABLE.iter_mut() {
        if !seg.in_use {
            seg.size_bytes = size;
            seg.phys_frame = 0x70000000 + (seg.id as u64 * 0x1000);
            seg.attach_count = 0;
            seg.owner_pid = 1;
            seg.in_use = true;
            return Ok(seg.id as usize);
        }
    }
    Err("SHM table full")
}

/// Remove and deallocate a shared memory segment by ID.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn remove_shm(id: u32) -> Result<(), &'static str> {
    for seg in SHM_TABLE.iter_mut() {
        if seg.id == id && seg.in_use {
            seg.in_use = false;
            seg.attach_count = 0;
            seg.size_bytes = 0;
            seg.phys_frame = 0;
            return Ok(());
        }
    }
    Err("Shared memory segment not found or not in use")
}

/// Create a new semaphore with given key and initial value.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn create_sem(key: u32, init_val: i32) -> Result<u32, &'static str> {
    for sem in SEM_TABLE.iter_mut() {
        if !sem.in_use {
            sem.key = key;
            sem.value = init_val;
            sem.waiters = 0;
            sem.in_use = true;
            return Ok(sem.id);
        }
    }
    Err("Semaphore table full")
}

/// Remove and deallocate a semaphore by ID.
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn remove_sem(id: u32) -> Result<(), &'static str> {
    for sem in SEM_TABLE.iter_mut() {
        if sem.id == id && sem.in_use {
            sem.in_use = false;
            sem.value = 0;
            sem.waiters = 0;
            return Ok(());
        }
    }
    Err("Semaphore not found or not in use")
}

/// Retrieve physical page frame for attached shared memory segment (Syscall 29: shmat).
///
/// # Safety
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn get_shm_frame(shmid: usize) -> Option<u64> {
    if shmid < 4 && SHM_TABLE[shmid].in_use {
        SHM_TABLE[shmid].attach_count += 1;
        Some(SHM_TABLE[shmid].phys_frame)
    } else {
        None
    }
}
