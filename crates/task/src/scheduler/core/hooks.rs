// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task cleanup hooks and procfs dynamic information providers.

use core::fmt::Write;

use super::state::{CURRENT_TASK_IDX, TASKS};
use crate::types::{TaskState, MAX_TASKS};

pub type TaskResourceCleanupHook = fn(pid: usize);
pub static mut TASK_CLEANUP_HOOK: Option<TaskResourceCleanupHook> = None;

/// Register kernel-level resource cleanup callback (invoked on task exit and zombie reaping).
pub fn register_task_cleanup_hook(hook: TaskResourceCleanupHook) {
    unsafe {
        TASK_CLEANUP_HOOK = Some(hook);
    }
}

/// Dynamic task status provider for /system/proc/[pid]/status.
pub fn task_status_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let mut open_fds = 0;
        for fd in &task.fds {
            if fd.is_open {
                open_fds += 1;
            }
        }
        let state_str = match task.state {
            TaskState::Running => "R (running)",
            TaskState::Ready => "S (sleeping)",
            TaskState::Blocked => "D (disk sleep)",
            TaskState::Zombie(_) | TaskState::Exited(_) => "Z (zombie)",
            TaskState::Created => "I (idle)",
        };

        struct SliceWriter<'a> {
            buf: &'a mut [u8],
            offset: usize,
        }
        impl<'a> Write for SliceWriter<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let avail = self.buf.len().saturating_sub(self.offset);
                let to_write = bytes.len().min(avail);
                self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
                self.offset += to_write;
                Ok(())
            }
        }

        let mut writer = SliceWriter { buf, offset: 0 };
        let _ = core::write!(
            writer,
            "Name:\t{}\n\
             State:\t{}\n\
             Tgid:\t{}\n\
             Pid:\t{}\n\
             PPid:\t{}\n\
             Uid:\t{}\t{}\n\
             Gid:\t{}\t{}\n\
             FDSize:\t{}\n\
             SigBlk:\t{:08x}\n\
             SigPnd:\t{:08x}\n",
            task.name,
            state_str,
            task.id,
            task.id,
            task.parent_id,
            task.uid,
            task.euid,
            task.gid,
            task.egid,
            open_fds,
            task.signal_mask,
            task.pending_signals
        );
        Some(writer.offset)
    }
}

/// Dynamic task cmdline provider for /system/proc/[pid]/cmdline.
pub fn task_cmdline_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let bytes = task.name.as_bytes();
        let to_copy = bytes.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[..to_copy]);
        Some(to_copy)
    }
}

/// Dynamic provider for active task PID.
pub fn current_pid_provider() -> usize {
    unsafe { CURRENT_TASK_IDX }
}
