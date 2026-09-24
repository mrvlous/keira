// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task lifecycle transitions, termination, and process synchronization.

pub mod exit;
pub mod reap;
pub mod wait;

pub use exit::{exit_current, stop_task};
pub use reap::{reap_orphaned_zombies, reap_orphaned_zombies_locked};
pub use wait::{sys_waitpid, wait_for_task};
