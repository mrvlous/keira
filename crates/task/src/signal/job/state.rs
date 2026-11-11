// SPDX-License-Identifier: GPL-2.0-only

//! Job control execution states and process session metadata.

/// Process lifecycle state within interactive job control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Terminated,
}

/// Metadata describing an active job control session entry.
#[derive(Debug, Clone, Copy)]
pub struct JobInfo {
    pub job_id: u32,
    pub pid: u32,
    pub name: [u8; 32],
    pub name_len: usize,
    pub state: JobState,
    pub is_foreground: bool,
}

/// Maximum number of tracked jobs in the job control table.
pub const MAX_JOBS: usize = 64;
