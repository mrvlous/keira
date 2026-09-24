// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Container PID namespace isolation and control group hierarchy table.

pub mod pid;
pub mod table;

pub use pid::translate_pid_to_namespace;
pub use table::{
    create_cgroup, delete_cgroup, get_cgroup_stats, get_cgroup_table, init, CGROUP_TABLE,
    NEXT_CGROUP_ID,
};
