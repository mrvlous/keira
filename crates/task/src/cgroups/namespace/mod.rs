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

//! Container PID namespace isolation and control group hierarchy table.

pub mod pid;
pub mod table;

pub use pid::translate_pid_to_namespace;
pub use table::{
    create_cgroup, delete_cgroup, get_cgroup_stats, get_cgroup_table, init, CGROUP_TABLE,
    NEXT_CGROUP_ID,
};
