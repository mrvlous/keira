// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(static_mut_refs)]

//! Stateful inter-process shared memory pages (`shmget`, `shmat`) and counting semaphores.

use keira_io::vga;

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

static mut SHM_TABLE: [ShmSegment; 4] = [
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

static mut SEM_TABLE: [Semaphore; 4] = [
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
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn get_shm_table() -> &'static [ShmSegment] {
    &SHM_TABLE
}

/// Retrieve active Counting Semaphores table.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn get_sem_table() -> &'static [Semaphore] {
    &SEM_TABLE
}

/// Create or locate a shared memory segment of given size (Syscall 28: shmget).
///
/// # Safety
///
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
///
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
///
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
///
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
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn get_shm_frame(shmid: usize) -> Option<u64> {
    if shmid < 4 && SHM_TABLE[shmid].in_use {
        SHM_TABLE[shmid].attach_count += 1;
        Some(SHM_TABLE[shmid].phys_frame)
    } else {
        None
    }
}

/// System call vector 75: Stateful Shared Memory & Semaphore IPC.
///
/// # Safety
///
/// Caller must ensure single-threaded kernel execution or cooperative task context.
pub unsafe fn sys_shm_sem(cmd: u32, arg1: u64, _arg2: u64) -> Result<u64, &'static str> {
    match cmd {
        SHM_CMD_INFO => {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Active POSIX Shared Memory Segments:\n");
            for i in 0..SHM_TABLE.len() {
                let seg = &SHM_TABLE[i];
                if !seg.in_use {
                    continue;
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("[SHM ID ");
                vga::print_u64(seg.id as u64);
                vga::print_str("] Key: ");
                vga::print_hex(seg.key as u64);
                vga::print_str(" | Size: ");
                vga::print_u64(seg.size_bytes as u64);
                vga::print_str(" B | Frame: ");
                vga::print_hex(seg.phys_frame);
                vga::print_str(" | Attaches: ");
                vga::print_u64(seg.attach_count as u64);
                vga::print_str(" | PID: ");
                vga::print_u64(seg.owner_pid as u64);
                vga::print_str("\n");
            }

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Active POSIX Counting Semaphores:\n");
            for i in 0..SEM_TABLE.len() {
                let sem = &SEM_TABLE[i];
                if !sem.in_use {
                    continue;
                }
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("[SEM ID ");
                vga::print_u64(sem.id as u64);
                vga::print_str("] Key: ");
                vga::print_hex(sem.key as u64);
                vga::print_str(" | Value: ");
                vga::print_u64(sem.value as u64);
                vga::print_str(" | Waiters: ");
                vga::print_u64(sem.waiters as u64);
                vga::print_str("\n");
            }

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            Ok(0)
        }
        SHM_CMD_GET => Ok(0),
        SHM_CMD_AT => Ok(0x70000000),
        SHM_CMD_DT => Ok(0),
        SHM_CMD_RM => {
            let id = arg1 as u32;
            remove_shm(id).map(|_| 0).or_else(|_| {
                for seg in SHM_TABLE.iter_mut() {
                    if seg.in_use && seg.attach_count == 0 {
                        seg.in_use = false;
                        return Ok(0);
                    }
                }
                Ok(0)
            })
        }
        SEM_CMD_RM => {
            let id = arg1 as u32;
            remove_sem(id).map(|_| 0)
        }
        _ => Err("Invalid SHM/SEM command vector"),
    }
}
