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

//! Signal job control table and task state tracking.

pub mod state {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum JobState {
        Running,
        Stopped,
        Terminated,
    }
    
    #[derive(Debug, Clone, Copy)]
    pub struct JobInfo {
        pub job_id: u32,
        pub pid: u32,
        pub name: [u8; 32],
        pub name_len: usize,
        pub state: JobState,
        pub is_foreground: bool,
    }
    
    pub const MAX_JOBS: usize = 64;
}

pub mod table {
    use super::state::{JobInfo, JobState, MAX_JOBS};
    
    pub static mut JOB_TABLE: [Option<JobInfo>; MAX_JOBS] = [const { None }; MAX_JOBS];
    pub static mut JOB_COUNT: usize = 0;
    
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
    
    pub unsafe fn get_foreground_job_pid() -> Option<u32> {
        for job in JOB_TABLE.iter().take(JOB_COUNT).flatten() {
            if job.is_foreground && job.state == JobState::Running {
                return Some(job.pid);
            }
        }
        None
    }
    
    pub unsafe fn remove_job_by_pid(pid: u32) {
        for (i, job_slot) in JOB_TABLE.iter_mut().enumerate().take(JOB_COUNT) {
            if let Some(ref job) = job_slot {
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
}

pub use self::state::*;
pub use self::table::*;
