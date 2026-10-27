// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic ProcFS pseudo-filesystem provider (`/proc/*`).
//!
//! Subdivided into specialized hyper-modular sub-packages:
//! - `writer/`: Fixed-slice formatting buffer implementing `core::fmt::Write`.
//! - `system/`: System-level telemetry (`uptime`, `meminfo`, `cpuinfo`, `version`, `loadavg`).
//! - `task/`: Process status and command-line arguments inspection.
//! - `dispatcher/`: Path routing and node existence query engine.

pub mod dispatcher;
pub mod system;
pub mod task;
pub mod writer;

#[cfg(test)]
mod tests;

pub use self::dispatcher::{
    exists, get_pid_dir_entry, get_proc_entry, is_proc_dir, read_proc_file, PID_DIR_ENTRIES,
    PROC_ENTRIES,
};
pub use self::system::{
    register_aslr_hook, register_net_hooks, AslrPolicyProvider, NetTelemetryProvider, NET_ARP_HOOK,
    NET_CONNTRACK_HOOK, NET_DEV_HOOK, NET_IPTABLES_HOOK, NET_RAW_HOOK, NET_ROUTE_HOOK,
    NET_SOCKSTAT_HOOK, NET_TCP_HOOK, NET_UDP_HOOK,
};
pub use self::task::{
    get_active_pids, read_task_cmdline, read_task_stat, read_task_statm, read_task_status,
    register_extended_task_hooks, register_task_hooks, task_exists, CurrentPidProvider,
    TaskCmdlineProvider, TaskExistsProvider, TaskListProvider, TaskStatProvider, TaskStatmProvider,
    TaskStatusProvider, CURRENT_PID_HOOK, TASK_CMDLINE_HOOK, TASK_EXISTS_HOOK, TASK_LIST_HOOK,
    TASK_STATM_HOOK, TASK_STATUS_HOOK, TASK_STAT_HOOK,
};
pub use self::writer::BufWriter;
