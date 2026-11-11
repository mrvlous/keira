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

//! Process memory limit enforcement, CPU slice accounting and container PID namespaces.

pub mod model;
pub mod namespace;
pub mod quota;

#[cfg(test)]
mod tests;

pub use model::{Cgroup, MAX_CGROUPS};
pub use namespace::{
    create_cgroup, delete_cgroup, get_cgroup_stats, get_cgroup_table, init,
    translate_pid_to_namespace, CGROUP_TABLE, NEXT_CGROUP_ID,
};
pub use quota::{check_memory_limit, set_cgroup_limits};
