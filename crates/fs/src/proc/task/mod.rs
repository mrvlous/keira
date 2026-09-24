// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process-level status and command-line arguments inspection in ProcFS.

pub mod cmdline;
pub mod hooks;
pub mod status;

pub use self::cmdline::read_task_cmdline;
pub use self::hooks::{
    register_task_hooks, CurrentPidProvider, TaskCmdlineProvider, TaskStatusProvider,
    CURRENT_PID_HOOK, TASK_CMDLINE_HOOK, TASK_STATUS_HOOK,
};
pub use self::status::read_task_status;
