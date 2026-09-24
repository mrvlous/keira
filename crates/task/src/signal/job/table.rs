// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global job control table and session membership tracking.

use super::state::{JobInfo, JobState, MAX_JOBS};

pub static mut JOB_TABLE: [Option<JobInfo>; MAX_JOBS] = [const { None }; MAX_JOBS];
pub static mut JOB_COUNT: usize = 0;

/// Register a new background or foreground process job into Job Control Table.
///
/// # Safety
/// The caller must ensure synchronized access to `JOB_TABLE` and `JOB_COUNT`.
pub unsafe fn add_job(pid: u32, name: &str, is_fg: bool) -> u32 {
    let job_id = (JOB_COUNT + 1) as u32;
    let nbytes = name.as_bytes();
    let len = if nbytes.len() > 32 { 32 } else { nbytes.len() };

    let mut info = JobInfo {
        job_id,
        pid,
        name: [0u8; 32],
        name_len: len,
        state: JobState::Running,
        is_foreground: is_fg,
    };
    info.name[..len].copy_from_slice(&nbytes[..len]);

    if JOB_COUNT < MAX_JOBS {
        JOB_TABLE[JOB_COUNT] = Some(info);
        JOB_COUNT += 1;
    } else {
        JOB_TABLE[0] = Some(info);
    }
    job_id
}

/// Query the currently running foreground job PID, if any.
///
/// # Safety
/// The caller must ensure synchronized access to `JOB_TABLE` and `JOB_COUNT`.
pub unsafe fn get_foreground_job_pid() -> Option<u32> {
    for i in 0..JOB_COUNT {
        if let Some(ref job) = JOB_TABLE[i] {
            if job.is_foreground && job.state == JobState::Running {
                return Some(job.pid);
            }
        }
    }
    None
}

/// Remove a job from the table by process ID.
///
/// # Safety
/// The caller must ensure synchronized access to `JOB_TABLE` and `JOB_COUNT`.
pub unsafe fn remove_job_by_pid(pid: u32) {
    for i in 0..JOB_COUNT {
        if let Some(ref job) = JOB_TABLE[i] {
            if job.pid == pid {
                JOB_TABLE[i] = None;
                for j in i..(JOB_COUNT.saturating_sub(1)) {
                    JOB_TABLE[j] = JOB_TABLE[j + 1];
                }
                if JOB_COUNT > 0 {
                    JOB_TABLE[JOB_COUNT - 1] = None;
                    JOB_COUNT -= 1;
                }
                break;
            }
        }
    }
}
