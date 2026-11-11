// SPDX-License-Identifier: GPL-2.0-only

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
