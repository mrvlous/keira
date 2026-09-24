// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic ProcFS pseudo-filesystem provider (`/system/proc/*`).
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

pub use self::dispatcher::{exists, read_proc_file};
pub use self::task::{
    register_task_hooks, CurrentPidProvider, TaskCmdlineProvider, TaskStatusProvider,
    CURRENT_PID_HOOK, TASK_CMDLINE_HOOK, TASK_STATUS_HOOK,
};
pub use self::writer::BufWriter;
