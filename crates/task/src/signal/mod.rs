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

//! POSIX signal generation, delivery, job control and process termination handling.

pub mod action;
pub mod dispatch;
pub mod job;
pub mod posix;

#[cfg(test)]
mod tests;

pub use action::{
    get_signal_handler, reset_signal_handlers, sys_sigaction, MAX_SIGNAL_TASKS, SIGNAL_HANDLERS,
};
pub use dispatch::sys_kill;
pub use job::{
    add_job, get_foreground_job_pid, remove_job_by_pid, JobInfo, JobState, JOB_COUNT, JOB_TABLE,
    MAX_JOBS,
};
pub use posix as constants;
pub use posix::*;
