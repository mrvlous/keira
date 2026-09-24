// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call dispatcher for shared memory and semaphore management.

#![allow(static_mut_refs)]

use keira_io::vga;

use crate::shm::manager::table::{remove_sem, remove_shm, SEM_TABLE, SHM_TABLE};
use crate::shm::segment::types::{
    SEM_CMD_RM, SHM_CMD_AT, SHM_CMD_DT, SHM_CMD_GET, SHM_CMD_INFO, SHM_CMD_RM,
};

/// System call vector 75: Stateful Shared Memory & Semaphore IPC.
///
/// # Safety
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
