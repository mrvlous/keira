// SPDX-License-Identifier: GPL-2.0-only

//! Container PID namespace isolation and control group hierarchy table.

pub mod pid;
pub mod table;

pub use pid::translate_pid_to_namespace;
pub use table::{
    create_cgroup, delete_cgroup, get_cgroup_stats, get_cgroup_table, init, CGROUP_TABLE,
    NEXT_CGROUP_ID,
};
