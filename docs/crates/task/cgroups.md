<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Resource Control Groups (cgroups) & Namespaces

Documentation for cgroups in [`crates/task/src/cgroups.rs`](../../../crates/task/src/cgroups.rs).

## Features
- Memory limit enforcement per process group (e.g. 64MB default limit).
- CPU time slice bandwidth throttling.
- PID Namespace translation isolating process hierarchies between virtual containers.
- System Call: `sys_cgroup_control` (Syscall 39).
