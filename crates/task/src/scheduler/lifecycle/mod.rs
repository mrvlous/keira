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

//! Task lifecycle transitions, termination and process synchronization.

pub mod exit;
pub mod reap;
pub mod wait;

pub use exit::{exit_current, stop_task};
pub use reap::{reap_orphaned_zombies, reap_orphaned_zombies_locked};
pub use wait::{sys_waitpid, wait_for_task};
