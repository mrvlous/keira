// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides support for task management, process representation, context switching,
//! and cooperative multitasking scheduling.

pub mod scheduler;
pub mod types;

/// Resource Control Groups (cgroups) & PID Namespaces
pub mod cgroup;

/// Mandatory Access Control (MAC / SELinux) Security Engine
pub mod mac;

/// Seccomp BPF System Call Filter Engine
pub mod seccomp;

pub use scheduler::{exit_current, init, list_tasks, spawn};
pub use types::{InterruptContext, Task, TaskState};
