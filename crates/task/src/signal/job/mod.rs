// SPDX-License-Identifier: GPL-2.0-only

//! Process job control session and foreground group management.

pub mod state;
pub mod table;

pub use state::{JobInfo, JobState, MAX_JOBS};
pub use table::{add_job, get_foreground_job_pid, remove_job_by_pid, JOB_COUNT, JOB_TABLE};
