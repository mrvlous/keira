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

/// Fixed slice writer helper implementing core::fmt::Write.
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

/// Dynamic task status provider for /proc/[pid]/status.
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
        let vsize_kb = if task.program_break > task.program_break_start {
            (task.program_break.saturating_sub(task.program_break_start)
                + super::state::TASK_STACK_SIZE)
                / 1024
        } else {
            super::state::TASK_STACK_SIZE / 1024
        };
        let rss_kb = vsize_kb;

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
             VmSize:\t{} kB\n\
             VmRSS:\t{} kB\n\
             Threads:\t1\n\
             SigBlk:\t{:08x}\n\
             SigPnd:\t{:08x}\n\
             voluntary_ctxt_switches:\t{}\n",
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
            vsize_kb,
            rss_kb,
            task.signal_mask,
            task.pending_signals,
            task.switches
        );
        Some(writer.offset)
    }
}

/// Dynamic task cmdline provider for /proc/[pid]/cmdline.
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

/// Dynamic task accounting stat provider for /proc/[pid]/stat.
pub fn task_stat_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let state_char = match task.state {
            TaskState::Running => 'R',
            TaskState::Ready => 'S',
            TaskState::Blocked => 'D',
            TaskState::Zombie(_) | TaskState::Exited(_) => 'Z',
            TaskState::Created => 'I',
        };
        let pgrp = if task.tgid != 0 { task.tgid } else { task.id };
        let vsize = if task.program_break > task.program_break_start {
            task.program_break.saturating_sub(task.program_break_start)
                + super::state::TASK_STACK_SIZE
        } else {
            super::state::TASK_STACK_SIZE
        };
        let rss = (vsize + 4095) / 4096;

        let mut writer = SliceWriter { buf, offset: 0 };
        let _ = core::write!(
            writer,
            "{} ({}) {} {} {} 0 0 -1 4194304 0 0 0 0 {} {} 0 0 20 0 1 0 0 {} {} 18446744073709551615 4194304 8388608 {} {} 0 {} {} 0 0 0 0 0 17 0 0 0 0 0 0 {} {} {} 0 0 0 0 {}\n",
            task.id,
            task.name,
            state_char,
            task.parent_id,
            pgrp,
            task.cpu_ticks,
            task.switches,
            vsize,
            rss,
            task.rsp,
            task.rsp,
            task.pending_signals,
            task.signal_mask,
            task.program_break_start,
            task.program_break,
            task.program_break_start,
            task.exit_code
        );
        Some(writer.offset)
    }
}

/// Dynamic task memory page metrics provider for /proc/[pid]/statm.
pub fn task_statm_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let vsize = if task.program_break > task.program_break_start {
            task.program_break.saturating_sub(task.program_break_start)
                + super::state::TASK_STACK_SIZE
        } else {
            super::state::TASK_STACK_SIZE
        };
        let total_pages = (vsize + 4095) / 4096;
        let rss_pages = total_pages;
        let shared_pages = 0;
        let text_pages = if task.is_user { 4 } else { 1 };
        let lib_pages = 0;
        let data_pages =
            (task.program_break.saturating_sub(task.program_break_start) + 4095) / 4096;
        let dt_pages = 0;

        let mut writer = SliceWriter { buf, offset: 0 };
        let _ = core::write!(
            writer,
            "{} {} {} {} {} {} {}\n",
            total_pages,
            rss_pages,
            shared_pages,
            text_pages,
            lib_pages,
            data_pages,
            dt_pages
        );
        Some(writer.offset)
    }
}

/// Checks whether a task exists in the global scheduler table.
pub fn task_exists_provider(pid: usize) -> bool {
    unsafe {
        if pid >= MAX_TASKS {
            return false;
        }
        TASKS[pid].is_some()
    }
}

/// Enumerates all active task PIDs into the provided buffer.
pub fn task_list_provider(out_pids: &mut [usize]) -> usize {
    unsafe {
        let mut count = 0;
        for (i, task) in TASKS.iter().enumerate() {
            if task.is_some() {
                if count < out_pids.len() {
                    out_pids[count] = i;
                    count += 1;
                }
            }
        }
        count
    }
}

/// Dynamic provider for active task PID.
pub fn current_pid_provider() -> usize {
    unsafe { CURRENT_TASK_IDX }
}

/// Dynamic task resource limits provider for /proc/[pid]/limits.
pub fn task_limits_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        Some(crate::rlimit::format_task_limits(&task.rlimits, buf))
    }
}
