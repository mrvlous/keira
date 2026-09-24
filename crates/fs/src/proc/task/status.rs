// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic `/system/proc/[pid]/status` process state inspection node.

use super::hooks::TASK_STATUS_HOOK;

/// Reads task status information for the given process ID into the buffer.
pub fn read_task_status(pid: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { TASK_STATUS_HOOK } {
        if let Some(bytes) = hook(pid, buf) {
            return Ok(bytes);
        }
    }
    Err("Process not found")
}
