// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process job control session and foreground group management.

pub mod state;
pub mod table;

pub use state::{JobInfo, JobState, MAX_JOBS};
pub use table::{add_job, get_foreground_job_pid, remove_job_by_pid, JOB_COUNT, JOB_TABLE};
